use crate::app::novel::{ChapterRead, Novel, NovelContentAmount, NovelFile, NovelSettings, NovelStatus, NovelType};
use crate::app::settings::ChapterReadPreference;
use crate::app::NOVEL_UPDATE_COOLDOWN;
use crate::appop::AppOp;
use crate::ui::novel_list::{ListStatus, ID_COLUMN};

use crate::app::error::ErisError;
use crate::app::history::NovelHistoryItem;
use crate::appop::messages::SortingMessage;
use crate::appop::novel_recognition::NovelRecognitionData;
use crate::appop::parsers::NovelParser;
use crate::ui::new_dialog::guess_keyword;
use crate::utils::gtk::BuilderExtManualCustom;
use crate::{data_dir, DATA_IMAGE_DIR};
use anyhow::Context;
use chrono::Local;
use gtk::prelude::{NotebookExt, StackExt, TreeModelExt, TreeViewExt, WidgetExt};
use ngrammatic::{CorpusBuilder, Pad};
use select::document::Document;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::mem;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use ureq::{Agent, Error};
use url::Url;

impl AppOp {
    /// Updates the content read count in db and UI.
    ///
    /// Used by a message.
    ///
    /// - Volume
    /// - Chapter
    /// - Side story
    pub fn chapter_read(&mut self, chapter_read: ChapterRead) {
        // Duplicate messages can happen when rapidly calling this function with the
        // `chapter_read_message` so check if this is a dublicate
        if let Some(previous_chapter_read) = &self.previous_chapter_read {
            if previous_chapter_read == &chapter_read {
                return;
            }
        }

        let volume_num = chapter_read.volume;
        let chapter_num = chapter_read.chapter;
        let side_story_num = chapter_read.side;
        let mut novel = chapter_read.novel;
        let exact_num = chapter_read.exact_num;

        if (novel.settings.content_read.chapters - chapter_num).abs() < f32::EPSILON
            && novel.settings.content_read.volumes == volume_num
            && novel.settings.content_read.side_stories == side_story_num
        {
            return;
        }

        debug!("appop::chapter_read");

        self.ui.update_reading_now_volume(volume_num);
        self.ui.update_reading_now_chapter(chapter_num);
        self.ui.update_reading_now_side_stories(side_story_num);

        let data = NovelRecognitionData::new(volume_num, chapter_num, side_story_num, None, String::new(), false);

        // Updated novel instance with correct chapter number
        novel = self.reading_novel(&mut novel, &data, exact_num);
        self.ui.update_currently_reading(&novel);
    }

    /// NOVEL DIALOG
    ///
    /// Update novel from slug if it has been long enough since the last time.
    pub fn update_novel_from_slug(&mut self) {
        debug!("appop::update_novel_from_slug");

        let novel = self.ui.lists.active_novel.clone();
        if let Some(old_novel) = &novel {
            debug!(
                "{:?} > {:?}",
                old_novel.last_scrape + NOVEL_UPDATE_COOLDOWN,
                Local::now().timestamp()
            );
            // Do nothing if it hasn't been long enough since the last update
            if old_novel.last_scrape + NOVEL_UPDATE_COOLDOWN > Local::now().timestamp() {
                self.ui.notification_dialog(&format!(
                    "`{}` was not updated from url. It has not been long enough since the last update.",
                    &old_novel.title
                ));
                info!(
                    "Novel `{}` was not updated from url. It has not beed long enough since the last update.",
                    &old_novel.title
                );
                return;
            }

            // All good so go for it
            if let Some(mut novel) = self.update_novel(old_novel) {
                novel.last_scrape = Local::now().timestamp();
                novel.settings = old_novel.settings.clone();

                debug!("update novel, response ok -> novel: {:?}", novel);
                novel = self.update_novel_in_db(novel);
                self.ui.novel_dialog.update(&self.ui.builder, &novel);
                self.ui.lists.list_update(&novel);
                self.ui.filter.list_update(&novel);
                self.ui.lists.active_novel = Some(novel.clone());

                self.history_send(NovelHistoryItem::new_history_update_novel(&novel));

                self.ui.novel_dialog.update_stack.set_visible_child_name("page0");

                self.update_reading_now_novel_info(&novel);
            }
        }
    }

    /// Opens a dialog from which a new `Novel` can be created.
    /// Some fields are prefilled from data gotten from an epub file.
    pub fn add_novel_from_file(&mut self, file: PathBuf, novel_file: NovelFile) {
        debug!("appop::add_novel_from_file");

        self.ui.file_new_dialog.update(&self.ui.builder, &novel_file);
        self.ui.file_new_dialog.dialog.show();

        self.file_to_add_from = Some(file);
        self.novel_file_data = Some(novel_file);
    }

    /// OK response for the "new from file" dialog.
    /// Adds a new `Novel` into the db and UI.
    /// Updates history.
    ///
    /// If `update` is `true` then update the novel from url.
    pub fn add_novel_from_file_done(&mut self, novel_file: NovelFile, update: bool) {
        debug!("appop::add_novel_from_file_done");

        // Check if this novel is already in the db
        if let Some(novel) = self.get_by_id(novel_file.novel_string_id.clone()) {
            self.ui
                .notification_dialog(&format!("`{}` already exists.", novel.title));
            return;
        }

        // Use `self.novel_file_data` here since it's the one
        // that has the cover image data.
        // `self.novel_file_data` should never be `None` here.
        let image = if let Some(cover_data) = &self.novel_file_data.as_ref().unwrap().cover_data {
            let cover_ext = self.novel_file_data.clone().unwrap().cover_ext.unwrap();
            let cover_file_path = format!("{}/{}.{}", DATA_IMAGE_DIR, &novel_file.novel_string_id, cover_ext);
            let path = data_dir(&cover_file_path);
            // Create the cover file if it doesn't exist
            if !path.exists() {
                let f = File::create(path).context(ErisError::WriteToDisk).unwrap();
                let mut writer = BufWriter::new(f);
                writer.write_all(cover_data).expect("Cannot write cover image file");
            }

            vec![cover_file_path]
        } else {
            vec![]
        };

        let list_status = match novel_file.status_list_id.as_str() {
            "0" => ListStatus::Reading,
            "1" => ListStatus::PlanToRead,
            "2" => ListStatus::OnHold,
            "3" => ListStatus::Completed,
            "4" => ListStatus::Dropped,
            _ => ListStatus::Reading,
        };

        // Novel settings
        let novel_settings = NovelSettings {
            list_status,
            content_read: NovelContentAmount {
                volumes: 0,
                chapters: novel_file.chapters.read as f32,
                side_stories: 0,
            },
            file: self.file_to_add_from.clone(),
            ..Default::default()
        };

        let authors: Vec<String> = novel_file.authors.split(',').map(|t| t.trim().to_string()).collect();

        let genres: Vec<String> = novel_file.genres.split(',').map(|t| t.trim().to_string()).collect();

        let novel_to_add = Novel {
            id: novel_file.novel_string_id,
            title: novel_file.novel_title,
            image,
            alternative_titles: None,
            description: Some(novel_file.description),
            author: authors,
            artist: vec![],
            genre: genres,
            tags: vec![],
            novel_type: NovelType::WebNovel,
            original_language: "".to_string(),
            translated: None,
            content: NovelContentAmount {
                volumes: 0,
                chapters: novel_file.chapters.available as f32,
                side_stories: 0,
            },
            status: NovelStatus::Ongoing,
            year: 0000,
            original_publisher: vec![],
            english_publisher: vec![],
            source: None,
            slug: novel_file.slug,
            last_scrape: Local::now().timestamp(),
            settings: novel_settings.clone(),
        };

        if update {
            // try to update the novel straight away before adding to the db
            let updated_novel = self.update_novel(&novel_to_add);
            if let Some(mut novel) = updated_novel {
                // Use the previous novel settings
                novel.settings = novel_settings;
                // Save the updated novel to db
                self.add_novel_to_db(novel);
            } else {
                // Update failed for some reason so add the one from the file
                self.add_novel_to_db(novel_to_add);
            }
        } else {
            // No update wanted so add the one from the file
            self.add_novel_to_db(novel_to_add);
        }
    }

    /// Creates a new `Novel` from given `url` and other variables.
    pub fn add_novel(
        &mut self,
        url: [String; 2],
        list_state: String,
        content_read: NovelContentAmount,
        reading_url_str: String,
        keywords: Option<Vec<String>>,
        score: String,
    ) {
        debug!("appop::add_novel");

        let url_start = url[0].clone();
        let url_end = url[1].clone();

        let list_status = match list_state.as_str() {
            "0" => ListStatus::Reading,
            "1" => ListStatus::PlanToRead,
            "2" => ListStatus::OnHold,
            "3" => ListStatus::Completed,
            "4" => ListStatus::Dropped,
            _ => ListStatus::Reading,
        };

        // Do nothing if the url is empty
        if url_end.is_empty() {
            return;
        }

        let mut url = format!("{}{}", url_start, url_end);

        // Add slash and the end of the url if using novelupdates.com
        if url.contains("novelupdates.com") && !url.ends_with('/') {
            url += "/";
        }

        // If a novel with the same slug already exists then do nothing
        if self.get_by_slug(url.clone()).is_some() {
            return;
        }

        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(2))
            .timeout_write(Duration::from_secs(2))
            .build();

        // Get the data from the webpage.
        match agent.get(url.as_str()).call() {
            Ok(response) => {
                let document = Document::from(response.into_string().expect("Cannot String").as_str());

                let novel_parser = NovelParser::from_url(&url);
                if !novel_parser.is_supported() {
                    // Do nothing more if novel parser is not supported
                    warn!("Novel was not added because the Source URL {} it not supported.", url);
                    return;
                }

                let reading_url = if let Ok(url) = Url::from_str(&reading_url_str) {
                    Some(url.to_string())
                } else {
                    None
                };

                if let Some(mut novel) = novel_parser.parse(document, &url) {
                    // Add the novel settings
                    novel.settings = NovelSettings {
                        list_status,
                        content_read,
                        notes: None,
                        score,
                        rereading: false,
                        reading_url,
                        window_titles: keywords,
                        file: None,
                        last_updated: Local::now().timestamp(),
                    };

                    // Add novel to db and UI
                    self.add_novel_to_db(novel);
                } else {
                    self.ui.notification_dialog("Could not add novel.");
                }
            }
            Err(Error::Status(code, _response)) => {
                self.ui.notification_dialog(&format!(
                    "Could not add novel!\n\nUrl {} returned {}",
                    url.as_str(),
                    code
                ));
                error!("Could not add novel. Url {} returned {}", url.as_str(), code);
                /* the server returned an unexpected status
                code (such as 400, 500 etc) */
            }
            Err(_) => { /* some kind of io/transport error */ }
        }
    }

    /// Handles moving a `Novel` to another list
    pub fn move_novel(&mut self, novel_id: String, to_list: ListStatus) {
        if let Some(mut novel) = self.get_by_id(novel_id) {
            // Create a copy of the novel settings and update the list status
            let mut new_novel_settings = novel.settings.clone();
            new_novel_settings.list_status = to_list;
            // Add the new novel settings to the novel
            novel.settings = new_novel_settings;
            // Save to db
            novel = self.update_novel_in_db(novel.clone());
            // Update the lists
            let iter = self.ui.lists.active_iter.take();
            self.ui.lists.list_move(&novel, iter);
            // Add a history entry
            self.history_send(NovelHistoryItem::new_history_novel_list_change(&novel));
        } else {
            // Practically impossible
            error!("Tried to move a novel but it could not be found anymore?!");
        }
    }

    /// NOVEL DIALOG
    ///
    /// Updates given `Novel` from URL, which is obtained from `Novel.slug`.
    pub fn update_novel(&mut self, old_novel: &Novel) -> Option<Novel> {
        // Do nothing if slug doesn't exist.
        if old_novel.slug.is_none() || old_novel.slug.as_ref().unwrap().is_empty() {
            self.ui.notification_dialog("Cannot update novel data, missing Url.");
            warn!("Cannot update novel data. Missing Url.");
            return None;
        }

        let url = old_novel.slug.as_ref().unwrap();

        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(2))
            .timeout_write(Duration::from_secs(2))
            .build();

        // Get the data from the webpage.
        match agent.get(url.as_str()).call() {
            Ok(response) => {
                debug!("update_novel, response ok");
                let document = Document::from(response.into_string().expect("Cannot String").as_str());

                let novel_parser = NovelParser::from_url(url);
                if !novel_parser.is_supported() {
                    // Do nothing more if novel parser is not supported
                    warn!(
                        "Novel {} was not updated because the Source URL {} it not supported.",
                        old_novel.title, url
                    );
                    return None;
                }

                return novel_parser.parse(document, url);
            }
            Err(Error::Status(code, _response)) => {
                self.ui.notification_dialog(&format!(
                    "Could not update novel!\n\nUrl {} returned {}",
                    url.as_str(),
                    code
                ));
                error!("Could not update novel! URL: {} returned {}", url.as_str(), code);
                /* the server returned an unexpected status
                code (such as 400, 500 etc) */
            }
            Err(_) => { /* some kind of io/transport error */ }
        }

        None
    }

    /// NOVEL DIALOG
    ///
    /// Deletes the novel from db and UI.
    pub fn delete_novel(&mut self) {
        if self.ui.lists.active_iter.is_none() {
            debug!("Cannot delete, no iter");
            return;
        }

        if let Some(novel) = self.ui.lists.active_novel.clone() {
            // Remove novel from the lists
            self.ui.filter.list_remove(&novel);
            self.ui.lists.list_remove(&novel, None);

            // Remove novel from the db
            let db = self.db.read().clone();
            if let Some(mut novels) = db.novels {
                let index = novels.iter().position(|n| n.id == novel.id).unwrap();

                novels.remove(index);

                self.db.write().novels = Some(novels);
                self.save_db_to_file();
                // Novel was deleted so add an entry into the history
                self.history_send(NovelHistoryItem::new_history_delete_novel(&novel));
                // let item = self.history.read().clone().new_history_delete_novel(&novel);
                // self.ui.history.list_insert(item);
            }
        }
    }

    /// NOVEL DIALOG
    ///
    /// Update novel settings.
    pub fn edit_novel_settings(&mut self, mut novel_settings: NovelSettings) {
        debug!("appop::edit_novel_settings | {:?}", novel_settings);

        if let Some(mut novel) = self.ui.lists.active_novel.clone() {
            // Check if the novel was moved to another list
            let old_iter = if novel.settings.list_status != novel_settings.list_status {
                self.ui.lists.find_iter(&novel)
            } else {
                None
            };

            // Remove any empty elements from the `window_titles` keyword vector
            if let Some(ref mut keywords) = novel_settings.window_titles {
                keywords.retain(|k| !k.is_empty());
            }

            // Refrain from updating the `last_updated` value when manually
            // editing the novel settings or the novels in the list
            // jump around if they are sorted by last updated.
            // (Take the old value and use it in the new novel settings)
            let last_updated = novel.settings.last_updated;
            novel.settings = novel_settings;
            novel.settings.last_updated = last_updated;
            novel = self.update_novel_in_db(novel.clone());

            if old_iter.is_some() {
                self.history_send(NovelHistoryItem::new_history_novel_list_change(&novel));

                self.ui.lists.list_move(&novel, old_iter);
            }

            self.ui.lists.list_update(&novel);
            self.ui.filter.list_update(&novel);
        }
    }

    /// Edit novel.
    pub fn edit_active_novel(&mut self) {
        debug!("appop::edit_active_novel");

        let novel = self.ui.lists.active_novel.clone().unwrap();
        let mut updated_novel = self.ui.novel_dialog.update_novel_from_edit(&self.ui.builder, &novel);

        updated_novel = self.update_novel_in_db(updated_novel.clone());

        self.ui.novel_dialog.update(&self.ui.builder, &updated_novel);
        self.ui.lists.list_update(&updated_novel);
        self.ui.filter.list_update(&novel);

        self.update_reading_now_novel_info(&updated_novel);

        self.ui.lists.active_novel = Some(updated_novel);
    }

    /// Update information on the reading now view for the current novel being visible there, if any.
    pub fn update_reading_now_novel_info(&mut self, novel: &Novel) {
        if let Some(currently_reading) = self.currently_reading.novel.read().as_ref() {
            if currently_reading.id == novel.id {
                self.ui.update_reading_now(&Some(novel.clone()));
            }
        }
    }

    /// Update the given `Novel` read count in db.
    /// Update the UI to reflect the change.
    /// Insert the change into the history list.
    pub fn edit_novel_chapters_read_count(
        &mut self,
        mut novel: Novel,
        old_iter: Option<gtk::TreeIter>,
        chapter_title: Option<String>,
    ) -> Novel {
        debug!("appop::edit_novel_chapters_read_count");

        let move_novel = old_iter.is_some();
        novel.settings.last_updated = Local::now().timestamp();

        let novel = self.update_novel_in_db(novel);
        self.ui.lists.list_update(&novel);
        self.ui.filter.list_update(&novel);

        if move_novel {
            self.history_send(NovelHistoryItem::new_history_novel_list_change(&novel));

            self.ui.lists.list_move(&novel, old_iter);
        }

        // Add history item if any of the content read values are over zero
        if novel.settings.content_read.chapters > 0.0
            || novel.settings.content_read.side_stories > 0
            || novel.settings.content_read.volumes > 0
        {
            self.history_send(NovelHistoryItem::new_history_chapter_read(&novel, chapter_title));
        }
        novel
    }

    /// Add novel to the db and write the change to file.
    /// Afterwards insert it into the novel list (UI).
    pub fn add_novel_to_db(&mut self, novel: Novel) {
        debug!("appop::add_to_novel_list");

        // Save to db
        self.db.write().push_novel(novel.clone());
        match self.db.write().write_to_file() {
            Ok(_) => {}
            Err(e) => {
                // This should practically never happen since if
                // deserializing fails during app startup then
                // serializing will not happen (since app is no longer up).
                error!("Could not add novel to db file. {}", e);
                return;
            }
        }

        // Add history entry
        self.history_send(NovelHistoryItem::new_history_add_novel(&novel));

        // Switch to the list the novel is added to
        self.ui.list_notebook.set_page(novel.settings.list_status.to_i32());

        // Update the UI lists
        self.ui.filter.list_insert(&novel);
        self.ui.lists.list_insert(&novel);

        // Update currently reading things only if the novel added is "relevant"
        if self.currently_reading.title.read().as_ref().is_some()
            && self
                .currently_reading
                .title
                .read()
                .as_ref()
                .unwrap()
                .contains(&novel.title)
        {
            // Update reading now
            self.ui.update_reading_now(&Some(novel));

            // Reset the currently reading title so the reading now view gets updated
            let _ = self.currently_reading.title.write().take();

            // Reset currently reading delay thingy
            self.currently_reading.timestamp_take();
        }
    }

    /// Write db to file in another thread.
    pub fn save_db_to_file(&mut self) {
        debug!("appop::save_db_to_file");

        let db = self.db.clone();
        thread::spawn(move || {
            debug!("Saving db to file in a new thread!");
            match db.write().write_to_file() {
                Ok(_) => {}
                Err(e) => {
                    // This should practically never happen since if
                    // deserializing fails during app startup then
                    // serializing will not happen (since app is no longer up).
                    error!("Could not add novel to db file. {}", e);
                }
            }
        });
    }

    /// Write everything to files in another thread.
    pub fn save_to_file(&mut self) {
        debug!("appop::save_to_file");

        self.save_db_to_file();

        let history = self.history.clone();
        thread::spawn(move || {
            debug!("Saving history to file in a new thread!");
            match history.write().write_to_file() {
                Ok(_) => {}
                Err(e) => error!("Could not save history to file in another thread. {:?}", e),
            };
        });

        let settings = self.settings.clone();
        thread::spawn(move || {
            debug!("Saving settings to file in a new thread!");
            match settings.write().write_to_file() {
                Ok(_) => {}
                Err(e) => error!("Could not save settings to a file in another thread. {:?}", e),
            };
        });

        if self.settings.read().general.window_state_enabled {
            let window_state = self.window_state.clone();
            thread::spawn(move || {
                debug!("Saving window state to file in a new thread!");
                if let Some(window_state) = window_state.write().clone() {
                    match window_state.write_to_file() {
                        Ok(_) => {}
                        Err(e) => error!("Could not save window state to file in another thread. {:?}", e),
                    };
                }
            });
        }
    }

    /// Try to find `Novel` by `Novel.id` from the db.
    pub fn get_by_id(&self, novel_id: String) -> Option<Novel> {
        if let Some(novels) = &self.db.read().novels {
            for novel in novels {
                if novel.id == novel_id {
                    return Some(novel.clone());
                }
            }
        }

        None
    }

    /// Try to find `Novel` by `Novel.slug` from the db.
    pub fn get_by_slug(&self, novel_slug: String) -> Option<Novel> {
        if let Some(novels) = &self.db.read().novels {
            for novel in novels {
                if let Some(slug) = &novel.slug {
                    if slug == &novel_slug || novel_slug.contains(slug) {
                        return Some(novel.clone());
                    }
                }
            }
        }

        None
    }

    /// Try to find `Novel` by `Novel.title` or `Novel.settings.window_titles` from the db.
    pub fn find_novel_by_window_title(&self, window_title: &str) -> Option<Novel> {
        if let Some(novels) = &self.db.read().novels {
            let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();

            // First check if the window title is identical to any novel in the db
            let exact_match = self.get_by_title(window_title);
            if exact_match.is_some() {
                return exact_match;
            }

            // Then check for keywords in novel settings
            let keyword_match = self.get_by_novel_keywords(window_title);
            if keyword_match.is_some() {
                return keyword_match;
            }

            // Lasty do a high percentage match fuzzy search
            for novel in novels {
                corpus.add_text(&novel.title.to_lowercase());
            }
            if let Some(top_result) = corpus.search(window_title, 0.25).first() {
                if top_result.similarity > 0.97 {
                    return self.get_by_title(&top_result.text);
                } else {
                    debug!(
                        "{} (did you mean {}? [{:.0}% match])",
                        window_title,
                        top_result.text,
                        top_result.similarity * 100.0
                    );
                }
            }
        }

        None
    }

    /// Try to find `Novel` by `Novel.settings.window_titles` from the db.
    pub fn get_by_novel_keywords(&self, window_title: &str) -> Option<Novel> {
        if let Some(novels) = &self.db.read().novels {
            for novel in novels {
                if let Some(window_titles) = &novel.settings.window_titles {
                    // Get by exact match
                    if window_titles.iter().any(|i| !i.is_empty() && i == window_title) {
                        return Some(novel.clone());
                    }
                }
            }
        }

        None
    }

    /// Get `Novel` from db by its title
    pub fn get_by_title(&self, window_title: &str) -> Option<Novel> {
        if let Some(novels) = &self.db.read().novels {
            for novel in novels {
                if novel.title.to_lowercase() == window_title.to_lowercase() {
                    return Some(novel.clone());
                }
            }
        }

        None
    }

    /// Find any number of potential novels based on the supplied title.
    ///
    /// TODO: this can be made nicer, I'm sure.
    pub fn find_potential_novels(&self, title: &str) -> Vec<Novel> {
        let mut potentials = vec![];
        if let Some(novels) = &self.db.read().novels {
            for novel in novels {
                if let Some(window_titles) = &novel.settings.window_titles {
                    for title in title.split_whitespace() {
                        for wtitle in window_titles {
                            if !wtitle.is_empty() && wtitle.contains(title) && !potentials.contains(novel) {
                                // Add a potential novel into the array,
                                // only once though
                                potentials.push(novel.clone());
                            }
                        }
                    }
                } else {
                    let acronym_from_title = guess_keyword(&novel.title);
                    if acronym_from_title.contains(title) && !potentials.contains(novel) {
                        potentials.push(novel.clone());
                    }
                }
            }
        }

        potentials
    }

    /// Add a novel into the db and update it in memory and write to a file.
    pub fn update_novel_in_db(&mut self, novel: Novel) -> Novel {
        let db = self.db.read().clone();

        if let Some(mut novels) = db.novels {
            let index = novels.iter().position(|n| n.id == novel.id).unwrap();

            let _ = mem::replace(&mut novels[index], novel.clone());

            self.db.write().novels = Some(novels);
            self.save_db_to_file();
        }

        novel
    }

    /// Update currently reading UI elements
    /// Gets the latest novel with read action from history and
    /// updates the UI with that information if it was found in
    /// the database.
    pub fn currently_reading(&self) {
        if let Some(last_read) = self.history.read().find_last_read() {
            if let Some(novel) = self.get_by_id(last_read.novel_id) {
                self.ui.update_currently_reading(&novel);
            }
        }
    }

    /// Reading a novel so update the given `Novel` data and return it.
    pub fn reading_novel(&mut self, novel: &mut Novel, data: &NovelRecognitionData, exact_num: bool) -> Novel {
        debug!("appop:reading_novel");

        let volume_num = data.volume;
        let chapter_num = data.chapter;
        let side_story_num = data.side_story;

        // Handle updating the data in novel and list
        let novel_preference = self.settings.read().clone().novel_recognition.chapter_read_preference;
        // Used to set the chapter read number to either current one or current one - 1
        let read_modifier = match novel_preference {
            ChapterReadPreference::Current => 0.0,
            ChapterReadPreference::Previous => 1.0,
        };

        // If found a chapter title and it is already present in the history
        // then there is no need to do anything more
        if let Some(chapter_title) = &data.chapter_title {
            if let Some(history_item) = self.history.read().find_chapter_title(chapter_title) {
                if let Some(found_novel) = self.get_by_id(history_item.novel_id) {
                    if found_novel == novel.clone() {
                        return novel.clone();
                    }
                }
            }
        }

        let mut new_chapter_read_num = if exact_num {
            chapter_num
        } else if data.chapter == 0.0 && data.reading && data.chapter_title.is_some() {
            // Add 1 to the chapters read number since something
            // is being read but no valid chapter number was found
            novel.settings.content_read.chapters + 1.0
        } else {
            chapter_num - read_modifier
        };

        // Do not set chapter number below 0
        if new_chapter_read_num < 0.0 {
            new_chapter_read_num = 0.0;
        }

        let mut new_side_story_num = if exact_num {
            side_story_num
        } else {
            side_story_num - (read_modifier as i32)
        };

        // Do not set side story number below 0
        if new_side_story_num < 0 {
            new_side_story_num = 0;
        }

        // Do nothing if the chapter number being read is less or same as
        // currently saved chapter read number and same for volume
        if !exact_num
            && novel.settings.content_read.chapters >= new_chapter_read_num
            && novel.settings.content_read.volumes >= volume_num
            && novel.settings.content_read.side_stories >= new_side_story_num
        {
            debug!("Volume/Chapter/Side stories read count was too low!");
            return novel.clone();
        }

        // If the numbers are not set by user (`exact_num` = probably set by user)
        // then make sure that the automation does not go backwards in case of a derp
        if exact_num {
            novel.settings.content_read.chapters = new_chapter_read_num;
            novel.settings.content_read.volumes = volume_num;
            novel.settings.content_read.side_stories = new_side_story_num;
        } else {
            // Update the volume/chapter/side story number only if it is larger than the saved one
            if new_chapter_read_num > novel.settings.content_read.chapters {
                novel.settings.content_read.chapters = new_chapter_read_num;
            }

            if volume_num > novel.settings.content_read.volumes {
                novel.settings.content_read.volumes = volume_num;
            }

            if new_side_story_num > novel.settings.content_read.side_stories {
                novel.settings.content_read.side_stories = new_side_story_num;
            }
        }

        // Check if the novel status allows for automatic novel list status changing to `Completed`
        let can_complete = matches!(&novel.status, NovelStatus::Completed | NovelStatus::Dropped)
            || self.settings.read().novel_recognition.autocomplete_ongoing;

        // Check if the novel should be moved to "reading" status
        // Only move novel to reading list from plan to read list
        // Do not move if the new chapter number was manually changed
        let move_to_reading = novel.settings.list_status == ListStatus::PlanToRead && !exact_num;

        // Change the list status to `Completed` if all the chapters have been read.
        // Should never work for `OnGoing` novels.
        let is_completed = (volume_num >= novel.content.volumes)
            && (new_chapter_read_num >= novel.content.chapters && novel.content.chapters > 0.0)
            && (side_story_num >= novel.content.side_stories)
            && can_complete;

        // Edit chapter read count
        // Get the correct list based on the list status in novel settings
        let treeview_id = novel.settings.list_status.treeview_id();
        let treeview = self.ui.builder.get::<gtk::TreeView>(treeview_id);
        let model = treeview.model().unwrap();

        // Very lazy? method to find the correct iter so the correct list row can be updated
        for i in 0..(model.iter_n_children(None)) {
            // Check if this is the correct iter
            let iter = model.iter_from_string(&i.to_string());
            if let Some(iter) = iter {
                // If the novel id in the row matches with the novel id being read then the correct one was found.
                let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
                if novel_id == novel.id {
                    self.ui.lists.active_list = novel.settings.list_status;
                    // Check if the novel should be moved to another list
                    let old_iter =
                        if novel.settings.list_status != ListStatus::Completed && is_completed || move_to_reading {
                            Some(iter)
                        } else {
                            None
                        };

                    if move_to_reading {
                        novel.settings.list_status = ListStatus::Reading;
                    } else if is_completed {
                        novel.settings.list_status = ListStatus::Completed;
                    }

                    // Update the chapters read count
                    return self.edit_novel_chapters_read_count(novel.clone(), old_iter, data.chapter_title.clone());
                }
            }
        }

        novel.clone()
    }

    /// Read novel by either opening a file or a webpoge.
    ///
    /// Priority:
    /// 1. Local file
    /// 2. Webpage
    ///
    /// If neither is set then show the reading url entry with focus.
    pub fn read_novel(&mut self, novel: Novel) {
        debug!(
            "appop::read_novel | {:?} | {:?}",
            novel.settings.file, novel.settings.reading_url
        );
        if let Some(reader_file) = novel.settings.file.as_ref() {
            let page = novel.settings.content_read.chapters;
            // File set, opening it with external reader
            debug!("File set: {:?}, page: {:?}", reader_file, page);

            if let Some(program) = &self.settings.read().general.reader {
                let args: Vec<String> = self
                    .settings
                    .read()
                    .general
                    .reader_args
                    .split(' ')
                    .map(move |s| {
                        if s.contains('%') {
                            s.replace("%f", reader_file.to_str().unwrap())
                                .replace("%p", page.to_string().as_str())
                        } else {
                            s.to_string()
                        }
                    })
                    .collect();

                debug!("Program: {:?}, args: {:?}", program, args);

                let output = Command::new(program).args(args).spawn();

                match output {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            } else {
            }
        } else if novel.settings.reading_url.is_some() {
            // Try to generate a new url with the next chapter if supported
            // otherwise use the supplied url as is
            if let Some(url) = novel.settings.reading_url() {
                // Open the url with default browser
                debug!("Opening url: {:?}", url);
                if webbrowser::open(&url).is_ok() {
                    // OK o_o
                }
            }
        } else {
            // No reading file or url set so open the settings
            // and focus the reading_url entry.
            self.ui.show_novel_dialog_reading_settings(&novel);

            self.ui.lists.active_list = novel.settings.list_status;
            self.ui.lists.active_iter = self.ui.lists.find_iter(&novel);
            self.ui.lists.active_novel = Some(novel.clone());
        }
    }

    /// Save new list sort order into settings.
    /// Only used for novel lists, not history one as it's probably not needed.
    pub fn update_list_sort_order(&mut self, list_sort: SortingMessage) {
        let mut settings = self.settings.write();
        settings.list.list_sort_order[list_sort.list_index as usize] = list_sort.sorting;
    }

    /// Save the new column width into settings.
    pub fn update_list_column_width(&mut self, list_index: i32, col_id: i32, col_width: i32) {
        let mut settings = self.settings.write();
        // Get the column for the correct list and update its width value
        // or insert a new one if it doesn't exist
        if let Some(col) = settings.list.column_width[list_index as usize].get_mut(&col_id) {
            *col = col_width;
        } else {
            settings.list.column_width[list_index as usize].insert(col_id, col_width);
        }
    }

    /// Do things after changing the main notebook page.
    pub fn switched_main_notebook_page(&self, page: u32) {
        match page {
            0 => {}
            1 => {}
            2 => self.ui.history.scroll_to_top(),
            _ => {}
        }
    }

    /// Opens the previously selected tab.
    ///
    /// Used for returning from the filter tab.
    pub fn return_to_selected_page(&self) {
        self.ui.list_notebook.set_page(self.ui.filter.selected_page);
    }

    /// Push a new keyword into novel settings `window_titles`.
    pub fn update_novel_reading_keyword(&mut self, mut novel: Novel, keyword: String) {
        debug!("Updated novel keywords with {:?}", keyword);

        if let Some(keywords) = novel.settings.window_titles.as_mut() {
            // Update
            keywords.push(keyword);
        } else {
            // Add new
            novel.settings.window_titles = Some(vec![keyword]);
        }

        // Save the changes
        let _ = self.update_novel_in_db(novel.clone());
    }

    /// Updates novel status (not list status) and returns the updated novel.
    pub fn update_novel_status(&mut self, mut novel: Novel, status: NovelStatus) -> Novel {
        debug!("Updated novel status to: {:?}", status);

        // Do nothing if novel status is already set the same
        if novel.status == status {
            return novel;
        }

        // Set the status
        novel.status = status;

        // Save the changes and return the saved Novel
        self.update_novel_in_db(novel)
    }
}

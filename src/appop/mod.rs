use std::ffi::OsStr;
use std::sync::Arc;

use gtk::prelude::*;
use parking_lot::RwLock;

use crate::app::database::{read_database, Database};
use crate::app::history::{NovelHistory, NovelHistoryItem};
use crate::app::novel::{ChapterRead, Novel, NovelFile, ReadAmount};
use crate::app::settings::Settings;
use crate::app::window_state::WindowState;
use crate::app::AppRuntime;
use crate::appop::messages::SortingMessage;
use crate::appop::novel_recognition::NovelRecognition;
use crate::appop::parsers::novel_title_to_slug;
use crate::{ui, UPDATE_LINK};
use chrono::Local;
use epub::doc::EpubDoc;
use gtk::{ButtonsType, DialogFlags, MessageType};
use select::document::Document;
use select::predicate::Name;
use std::path::{Path, PathBuf};

pub mod history;
pub mod messages;
mod novel;
pub mod novel_recognition;
pub mod parsers;
pub mod settings;
mod update;

#[derive(Debug)]
pub struct CurrentlyReading {
    pub title: Arc<RwLock<Option<String>>>,
    pub novel: Arc<RwLock<Option<Novel>>>,
    pub timestamp: Arc<RwLock<Option<i64>>>,
    pub timestamp_used: bool,
}

impl CurrentlyReading {
    pub fn timestamp_spend(&self, delay: i64) -> bool {
        if let Some(timestamp) = *self.timestamp.read() {
            if Local::now().timestamp() >= timestamp + delay {
                return true;
            }
        }

        false
    }

    pub fn timestamp_exists(&self) -> bool {
        self.timestamp.read().is_some()
    }

    pub fn timestamp_set(&mut self) {
        self.timestamp.write().replace(Local::now().timestamp());
        self.timestamp_used = false;
    }

    pub fn timestamp_take(&mut self) {
        let _ = self.timestamp.write().take();
        self.timestamp_used = true;
    }
}

pub struct AppOp {
    pub window_state: Arc<RwLock<Option<WindowState>>>,
    pub app_runtime: AppRuntime,
    pub ui: ui::UI,
    pub settings: Arc<RwLock<Settings>>,
    pub db: Arc<RwLock<Database>>,
    pub history: Arc<RwLock<NovelHistory>>,
    pub currently_reading: CurrentlyReading,
    pub novel_recognition: Option<NovelRecognition>,

    pub chapter_read_sender: Option<glib::Sender<ChapterRead>>,
    pub history_sender: Option<glib::Sender<NovelHistoryItem>>,
    pub previous_chapter_read: Option<ChapterRead>,
    pub list_populated: bool,
    pub list_sort_sender: Option<glib::Sender<SortingMessage>>,

    pub file_to_add_from: Option<PathBuf>,
    pub novel_file_data: Option<NovelFile>,
}

impl AppOp {
    pub fn new(ui: ui::UI, app_runtime: AppRuntime) -> AppOp {
        debug!("appop::new");

        let settings_data = Settings::open().expect("Failed to open settings file.");
        let settings = Arc::new(RwLock::new(settings_data));
        let currently_reading = CurrentlyReading {
            title: Arc::new(RwLock::new(None)),
            novel: Arc::new(RwLock::new(None)),
            timestamp: Arc::new(RwLock::new(None)),
            timestamp_used: false,
        };

        let history = match NovelHistory::open() {
            Ok(history) => Arc::new(RwLock::new(history)),
            Err(e) => {
                panic!("{:?}", e);
            }
        };

        let targets = vec![gtk::TargetEntry::new(
            "text/uri-list",
            gtk::TargetFlags::OTHER_APP,
            0,
        )];

        ui.main_window
            .drag_dest_set(gtk::DestDefaults::ALL, &targets, gdk::DragAction::COPY);

        let app_runtime_clone = app_runtime.clone();
        ui.main_window
            .connect_drag_data_received(move |window, _, _, _, selection, _, _| {
                for file in selection.uris() {
                    let file = gio::File::for_uri(&file);
                    let file_name = if file.is_native() {
                        file.path().unwrap().display().to_string()
                    } else {
                        file.uri().into()
                    };

                    match Path::new(&file_name)
                        .extension()
                        .and_then(OsStr::to_str)
                        .unwrap()
                    {
                        "epub" => {
                            let epub_doc = EpubDoc::new(file_name.clone());
                            // If the epub file is not valid then display
                            // a notification dialog about that
                            match epub_doc {
                                Ok(mut doc) => {
                                    let novel_title = if let Some(t) = doc.mdata("title") {
                                        t
                                    } else {
                                        "?".to_string()
                                    };
                                    let authors = if let Some(t) = doc.mdata("creator") {
                                        t
                                    } else {
                                        "?".to_string()
                                    };
                                    let genres = if let Some(t) = doc.mdata("genres") {
                                        t
                                    } else {
                                        "?".to_string()
                                    };
                                    let page_count = doc.get_num_pages();

                                    let intro_key =
                                        doc.resources.iter().find_map(|(k, (kk, _))| {
                                            if kk
                                                .clone()
                                                .into_os_string()
                                                .into_string()
                                                .unwrap()
                                                .contains("ntro")
                                            {
                                                Some(k)
                                            } else {
                                                None
                                            }
                                        });

                                    let mut description = String::new();
                                    if let Some(key) = intro_key {
                                        // If the spine in epub file is missing the intro chapter file
                                        // then add it in if it exists
                                        if !doc.spine.contains(key) {
                                            doc.spine.insert(0, key.clone());
                                        }
                                        // If page number was found then the description can be populated
                                        if let Some(page) =
                                            doc.resource_id_to_chapter(intro_key.unwrap())
                                        {
                                            // Try to change the page
                                            match doc.set_current_page(page) {
                                                Ok(_) => {
                                                    // Could set the current page to the introduction page so
                                                    // get the contents of it
                                                    let intro_text = doc.get_current_str().ok();
                                                    // Hope the content is properly in a body-tag and
                                                    // get the text inside it for the `description` variable
                                                    let html = Document::from(
                                                        intro_text.unwrap().as_str(),
                                                    );
                                                    for body in html.select(Name("body")) {
                                                        description.push_str(body.text().as_str());
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("{}", e);
                                                }
                                            }
                                        } else {
                                            warn!("Intro text was not found in the epub file.");
                                        }
                                    }

                                    // Get cover image data
                                    let cover_data = if let Ok(cover) = doc.get_cover() {
                                        Some(cover)
                                    } else {
                                        None
                                    };

                                    // Get cover image extension if it exists
                                    let cover_ext = if cover_data.is_some() {
                                        let mime = doc
                                            .get_resource_mime(&doc.get_cover_id().unwrap())
                                            .unwrap();
                                        let cover_ext = if mime.contains("png") {
                                            "png".to_string()
                                        } else {
                                            "jpg".to_string()
                                        };
                                        Some(cover_ext)
                                    } else {
                                        None
                                    };

                                    let novel_file = NovelFile {
                                        novel_string_id: novel_title_to_slug(&novel_title),
                                        novel_title,
                                        authors,
                                        genres,
                                        description,
                                        chapters: ReadAmount::new(page_count as f64),
                                        status_list_id: "0".to_string(),
                                        slug: None,
                                        cover_data,
                                        cover_ext,
                                    };

                                    app_runtime_clone.update_state_with(move |state| {
                                        state.add_novel_from_file(
                                            Path::new(&file_name).to_path_buf(),
                                            novel_file,
                                        );
                                    });
                                }
                                Err(e) => {
                                    let msg = &fl!("epub-file-warning", err = e.to_string());
                                    // Cannot use the `ui.notification_dialog`
                                    // because moving `ui` in here is an issue as it is
                                    // needed afterwards
                                    let dialog = gtk::MessageDialog::new(
                                        Some(&window.to_owned()),
                                        DialogFlags::DESTROY_WITH_PARENT,
                                        MessageType::Warning,
                                        ButtonsType::Ok,
                                        msg,
                                    );
                                    dialog.show();
                                    dialog.run();
                                    {
                                        dialog.close();
                                        return;
                                    }
                                }
                            }
                        }
                        "json" => {
                            app_runtime_clone.update_state_with(move |state| {
                                match state.import_json_to_db(file_name) {
                                    Ok(_) => {
                                        state.ui.open_post_import_message(&state.app_runtime);
                                    }
                                    Err(e) => {
                                        error!("{:?}", e);
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                }
            });

        let db = Arc::new(RwLock::new(read_database()));

        AppOp {
            window_state: Arc::new(RwLock::new(None)),
            app_runtime,
            ui,
            settings,
            db,
            history,
            currently_reading,
            novel_recognition: None,
            chapter_read_sender: None,
            history_sender: None,
            previous_chapter_read: None,
            list_populated: false,
            list_sort_sender: None,
            file_to_add_from: None,
            novel_file_data: None,
        }
    }

    pub fn init(&mut self) {
        if let Some(novels) = self.db.read().novels.clone() {
            self.ui
                .lists
                .add_columns(self.app_runtime.clone(), &self.settings.read());
            self.ui.lists.populate_columns(&novels);

            self.ui
                .filter
                .add_columns(self.app_runtime.clone(), &self.settings.read());
            self.ui.filter.populate_columns(&novels);
        }

        self.ui.history.add_columns(&self.ui.builder);
        self.ui.history.populate_columns(&self.history.read().items);

        debug!("appop::init");

        let app_runtime = self.app_runtime.clone();
        self.novel_recognition =
            NovelRecognition::new(app_runtime, self.settings.read().novel_recognition.clone());
        self.chapter_read_sender = Some(self.chapter_read_message());
        self.history_sender = Some(self.history_message());
        self.list_sort_sender = Some(self.list_sort_message());

        self.currently_reading();

        self.list_populated = true;
    }

    pub fn quit(&self) {
        debug!("appop::quit");
    }

    pub fn open_update_link(&self) {
        if webbrowser::open(UPDATE_LINK).is_ok() {}
    }
}

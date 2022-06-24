use crate::app::novel::NovelContentAmount;
use crate::app::AppRuntime;
use crate::appop::parsers::{novel_title_to_slug, NovelParser};
use crate::utils::gtk::BuilderExtManualCustom;
use gtk::prelude::WidgetExtManual;
use gtk::prelude::*;
use gtk::{Dialog, ResponseType};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Debug)]
pub struct NewNovelDialog {
    pub dialog: Dialog,
    pub url_combobox: gtk::ComboBoxText,
    pub status_combobox: gtk::ComboBoxText,
}

impl NewNovelDialog {
    pub fn new(builder: &gtk::Builder, parent: &gtk::ApplicationWindow) -> NewNovelDialog {
        let dialog = cascade! {
            builder.get::<gtk::Dialog>("new_dialog");
            ..set_title(&format!("{} - Eris", &fl!("add-novel")));
            ..set_modal(true);
            ..set_transient_for(Some(parent));
        };

        // Translate all the things
        builder.label_i18n("new_novel_url_label", &(fl!("new-novel-url") + ":"));
        builder.label_i18n("new_reading_status_label", &(fl!("reading-status") + ":"));
        builder.label_i18n("new_volumes_read_label", &(fl!("volumes-read") + ":"));
        builder.label_i18n("new_chapters_read_label", &(fl!("chapters-read") + ":"));
        builder.label_i18n("new_side_stories_read_label", &(fl!("side-stories-read") + ":"));
        builder.label_i18n("new_novel_reading_url_label", &(fl!("new-novel-reading-url") + ":"));
        builder.label_i18n("new_rec_keywords_label", &(fl!("new_rec_keywords") + ":"));
        builder.label_i18n("new_score_label", &(fl!("column-score") + ":"));

        let url_combobox = builder.get::<gtk::ComboBoxText>("new_novel_url_combobox");
        let novel_list_status_comboboxtext = builder.get::<gtk::ComboBoxText>("novel_list_status_comboboxtext");
        let novel_volumes_read_spinbutton = builder.get::<gtk::SpinButton>("novel_volumes_read_spinbutton");
        let novel_chapters_read_spinbutton = builder.get::<gtk::SpinButton>("novel_chapters_read_spinbutton");
        let novel_side_stories_read_spinbutton = builder.get::<gtk::SpinButton>("novel_side_stories_read_spinbutton");

        novel_volumes_read_spinbutton.set_range(0.0, 1_000_000.0);
        novel_chapters_read_spinbutton.set_range(0.0, 1_000_000.0);
        novel_side_stories_read_spinbutton.set_range(0.0, 1_000_000.0);

        NewNovelDialog {
            dialog,
            url_combobox,
            status_combobox: novel_list_status_comboboxtext,
        }
    }

    pub fn connect(&self, builder: &gtk::Builder, app_runtime: AppRuntime, url_list: Vec<String>) {
        let novel_url_entry = builder.get::<gtk::Entry>("novel_url_entry");
        let url_combobox = builder.get::<gtk::ComboBoxText>("new_novel_url_combobox");
        let novel_list_status_comboboxtext = builder.get::<gtk::ComboBoxText>("novel_list_status_comboboxtext");
        let novel_volumes_read_spinbutton = builder.get::<gtk::SpinButton>("novel_volumes_read_spinbutton");
        let novel_chapters_read_spinbutton = builder.get::<gtk::SpinButton>("novel_chapters_read_spinbutton");
        let novel_side_stories_read_spinbutton = builder.get::<gtk::SpinButton>("novel_side_stories_read_spinbutton");
        let novel_keywords_entry = builder.get::<gtk::Entry>("novel_keywords_entry");
        let new_novel_score = builder.get::<gtk::ComboBoxText>("new_novel_score");
        let novel_reading_url_entry = builder.get::<gtk::Entry>("novel_reading_url_entry");

        let keywords_entry = novel_keywords_entry.clone();
        let url_list_clone = url_list.clone();
        novel_url_entry.connect_changed(move |entry| {
            for (index, url_in_list) in url_list_clone.iter().enumerate() {
                match Url::parse(&entry.text().to_string()) {
                    Ok(url) => {
                        if url.path().len() > 1 {
                            if entry.text().contains(&url_in_list.to_string()) {
                                // Display the correct combobox item
                                url_combobox.set_active(Some(index as u32));
                                // Update the url entry text by removing the domain name and such.
                                entry.set_text(
                                    &entry
                                        .text()
                                        .replace(&url_combobox.active_text().unwrap().to_string(), ""),
                                );
                                // Guess the keyword from the slug, probably wrong most of the time.
                                keywords_entry.set_text(&guess_keyword(&entry.text().to_string()));
                            } else {
                                // Set the combobox item to "unsupported/other"
                                url_combobox.set_active_id(Some("0"));
                            }
                        }
                    }
                    Err(_e) => {
                        if entry.text().to_string().contains(' ') {
                            entry.set_text(&entry.text().replace(' ', "-"));
                        }
                    }
                }
            }
        });

        // Hide the element instead of deleting it when the close button is clicked
        self.dialog
            .connect_delete_event(move |dialog, _event| dialog.hide_on_delete());

        self.dialog.connect_key_release_event(|d, e| {
            if let Some(keycode) = e.keycode() {
                // Send Ok response when Enter key is pressed
                if keycode == 13 {
                    d.response(ResponseType::Ok);
                }
            }
            gtk::Inhibit(false)
        });

        self.dialog.connect_response(glib::clone!(@strong app_runtime, @strong url_list, @weak self.url_combobox as url_combobox => move |dialog, response_type| {
            match response_type {
                gtk::ResponseType::Ok => {
                    dialog.hide();

                    let mut url_start = String::new();
                    let active_id = url_combobox.active_id().unwrap().parse::<i32>().unwrap() as usize;
                    if active_id > 0 {
                        url_start.push_str(url_list.get(active_id).unwrap());
                    }
                    let url_end: String = novel_url_entry.text().to_string();
                    let list_status: String = novel_list_status_comboboxtext.active_id().unwrap().parse().unwrap();
                    let volumes_read: i32 = novel_volumes_read_spinbutton.value_as_int();
                    let chapters_read: f32 = novel_chapters_read_spinbutton.value() as f32;
                    let side_stories_read: i32 = novel_side_stories_read_spinbutton.value_as_int();
                    let reading_url = novel_reading_url_entry.text().to_string();

                    let novel_keywords = if novel_keywords_entry.text().is_empty() {
                        None
                    } else {
                        let kw = novel_keywords_entry
                            .text()
                            .split(',')
                            .map(|t|t.trim().to_string())
                            .collect::<Vec<String>>();
                        Some(kw)
                    };

                    let score = new_novel_score.active_id().unwrap().to_string();

                    app_runtime.update_state_with(move |state| {
                        let content_read = NovelContentAmount {
                            volumes: volumes_read,
                            chapters: chapters_read,
                            side_stories: side_stories_read,
                        };
                        let url = [url_start, url_end];
                        state.add_novel(url, list_status, content_read, reading_url, novel_keywords, score);
                    });
                },
                gtk::ResponseType::Cancel => dialog.hide(),
                _ => ()
            }
        }));
    }

    pub fn update(&self, builder: &gtk::Builder) {
        let novel_url_entry = builder.get::<gtk::Entry>("novel_url_entry");
        let novel_volumes_read_spinbutton = builder.get::<gtk::SpinButton>("novel_volumes_read_spinbutton");
        let novel_chapters_read_spinbutton = builder.get::<gtk::SpinButton>("novel_chapters_read_spinbutton");
        let novel_keywords_entry = builder.get::<gtk::Entry>("novel_keywords_entry");

        let reading_novel_title_label = builder.get::<gtk::Label>("reading_novel_title_label");
        let reading_novel_source_label = builder.get::<gtk::Label>("reading_novel_source_label");
        let reading_volume_number = builder.get::<gtk::Label>("reading_volume_number");
        let reading_chapter_number = builder.get::<gtk::Label>("reading_chapter_number");
        let new_novel_score = builder.get::<gtk::ComboBoxText>("new_novel_score");
        let novel_reading_url_entry = builder.get::<gtk::Entry>("novel_reading_url_entry");

        let reading_notebook = builder.get::<gtk::Notebook>("reading_notebook");

        new_novel_score.set_active_id(Some("0.0"));

        // Doesn't care about the chapter read number preference setting
        let volumes_read = reading_volume_number.text().parse::<f64>().unwrap();
        let chapters_read = reading_chapter_number.text().parse::<f64>().unwrap_or(0.0);
        let mut list_status = Some("1");
        if chapters_read > 0.0 {
            list_status = Some("0");
        }

        let mut slug = String::new();
        let mut keyword = String::new();
        let mut url_combobox_id = Some("1");
        let source = reading_novel_source_label.text().to_string();
        let novel_parser = NovelParser::from_str(source.as_str()).unwrap();

        // Change the combobox selection based on the source
        // and default to novelupdates one
        match novel_parser {
            NovelParser::RoyalRoad => {
                url_combobox_id = Some("2");
            }
            NovelParser::ScribbleHub => {
                url_combobox_id = Some("3");
            }
            _ => {
                // Try to guess the correct slug from the novel title
                slug.push_str(&novel_title_to_slug(&reading_novel_title_label.text().to_string()));
                keyword.push_str(&guess_keyword(&reading_novel_title_label.text().to_string()));

                // Guess the keyword from the novel title
                keyword.push_str(&guess_keyword(&reading_novel_title_label.text().to_string()));
            }
        }

        self.status_combobox.set_active_id(list_status);
        novel_volumes_read_spinbutton.set_value(volumes_read);
        novel_chapters_read_spinbutton.set_value(chapters_read);
        novel_reading_url_entry.set_text("");

        if let Some(page) = reading_notebook.current_page() {
            if page == 0 {
                // Notebook page is 0 so nothing probably not reading anything, use default values
                self.url_combobox.set_active(Some(1));
                self.status_combobox.set_active_id(Some("1"));
                novel_url_entry.set_text("");
                novel_url_entry.grab_focus();
                novel_chapters_read_spinbutton.set_value(0.0);
                novel_volumes_read_spinbutton.set_value(0.0);
                novel_keywords_entry.set_text("");
            } else {
                // Reading something so pre-set some values
                self.url_combobox.set_active_id(url_combobox_id);
                novel_url_entry.set_text(&slug);
                novel_url_entry.grab_focus();
                novel_keywords_entry.set_text(&keyword);
            }
        }
    }
}

/// Turn the novel title into an acronym if it is long enough.
/// Only include capitalized characters in the title.
pub fn guess_keyword(title: &str) -> String {
    if title.len() > 7 {
        title.chars().filter(|s| s.is_ascii_uppercase()).collect()
    } else {
        title.to_string()
    }
}

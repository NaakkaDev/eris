use crate::app::novel::{NovelFile, ReadAmount};
use crate::app::AppRuntime;
use crate::utils::gtk::BuilderExtManualCustom;
use gdk::gdk_pixbuf::{InterpType, Pixbuf};
use gtk::prelude::WidgetExtManual;
use gtk::prelude::*;
use gtk::{Dialog, ResponseType};
use std::io::Cursor;
use url::Url;

#[derive(Clone, Debug)]
pub struct FileNewNovelDialog {
    pub dialog: Dialog,
    pub status_combobox: gtk::ComboBoxText,
    pub url_combobox: gtk::ComboBoxText,
    pub novel_file_data: Option<NovelFile>,
    pub update_from_url: gtk::CheckButton,
}

impl FileNewNovelDialog {
    pub fn new(builder: &gtk::Builder, parent: &gtk::ApplicationWindow) -> FileNewNovelDialog {
        let dialog = cascade! {
            builder.get::<gtk::Dialog>("file_new_dialog");
            ..set_title(&format!("{} - Eris", &fl!("add-novel-file")));
            ..set_transient_for(Some(parent));
        };

        // Translate all the things
        builder.label_i18n("new_file_title_label", &(fl!("title-label") + ":"));
        builder.label_i18n("new_file_authors_label", &(fl!("novel-author") + ":"));
        builder.label_i18n("new_file_genres_label", &(fl!("novel-genre") + ":"));
        builder.label_i18n("new_file_chapters_read_label", &(fl!("chapters-read") + ":"));
        builder.label_i18n("new_file_chapters_available_label", &(fl!("chapters-available") + ":"));
        builder.label_i18n("new_file_reading_status_label", &(fl!("reading-status") + ":"));
        builder.label_i18n("new_file_novel_url_label", &(fl!("novel-url") + ":"));
        builder.label_i18n("new_file_description_label", &(fl!("novel-description") + ":"));

        builder.checkbutton_i18n("file_new_novel_update", &fl!("update-from-url-checkbutton"));

        builder.button_i18n("file_new_btn_ok", &fl!("ok-button"));
        builder.button_i18n("file_new_btn_cancel", &fl!("cancel-button"));

        let reading_status = builder.get::<gtk::ComboBoxText>("file_new_reading_status");
        let file_new_novel_url = builder.get::<gtk::ComboBoxText>("file_new_novel_url");
        let update_from_url = builder.get::<gtk::CheckButton>("file_new_novel_update");

        FileNewNovelDialog {
            dialog,
            status_combobox: reading_status,
            url_combobox: file_new_novel_url,
            novel_file_data: None,
            update_from_url,
        }
    }

    pub fn connect(&self, builder: &gtk::Builder, app_runtime: AppRuntime, url_list: Vec<String>) {
        let novel_title = builder.get::<gtk::Entry>("file_new_novel_title");
        let novel_authors = builder.get::<gtk::Entry>("file_new_novel_authors");
        let novel_genres = builder.get::<gtk::Entry>("file_new_novel_genres");
        let novel_description = builder.get::<gtk::TextView>("file_new_novel_description");

        let novel_chapters_read = builder.get::<gtk::SpinButton>("file_new_chapters_read");
        let novel_chapters_available = builder.get::<gtk::SpinButton>("file_new_chapters_available");
        let novel_url_entry = builder.get::<gtk::Entry>("file_new_novel_url_entry");

        let url_combobox = self.url_combobox.clone();
        novel_url_entry.connect_changed(move |entry| {
            for (index, url_in_list) in url_list.iter().enumerate() {
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

        self.dialog.connect_response(glib::clone!(@strong app_runtime, @weak self.status_combobox as status_combobox, @weak self.url_combobox as url_combobox @weak self.update_from_url as update_from_url => move |dialog, response_type| {
            match response_type {
                gtk::ResponseType::Ok => {
                    dialog.hide();

                    let desc_buffer = novel_description.buffer().unwrap();
                    let desc = desc_buffer
                        .text(&desc_buffer.start_iter(), &desc_buffer.end_iter(), false)
                        .unwrap_or_else(|| "".into())
                        .to_string();

                    let url = format!("{}{}", url_combobox.active_text().unwrap(), novel_url_entry.text());

                    let novel_file_data = NovelFile {
                        novel_string_id: slug::slugify(novel_title.text().to_string().replace('’', "-").replace('\'', "-")),
                        novel_title: novel_title.text().to_string(),
                        authors: novel_authors.text().to_string(),
                        genres: novel_genres.text().to_string(),
                        description: desc,
                        chapters: ReadAmount {
                            read: novel_chapters_read.value(),
                            available: novel_chapters_available.value(),
                        },
                        status_list_id: status_combobox.active_id().unwrap().to_string(),
                        slug: Some(url),
                        cover_data: None,
                        cover_ext: None,
                    };

                    let update = update_from_url.is_active();
                    app_runtime.update_state_with(move |state| {
                        state.add_novel_from_file_done(novel_file_data, update);
                    });
                },
                gtk::ResponseType::Cancel => dialog.hide(),
                _ => ()
            }
        }));
    }

    pub fn update(&self, builder: &gtk::Builder, novel_file_data: &NovelFile) {
        let novel_title = builder.get::<gtk::Entry>("file_new_novel_title");
        let novel_authors = builder.get::<gtk::Entry>("file_new_novel_authors");
        let novel_genres = builder.get::<gtk::Entry>("file_new_novel_genres");
        let novel_description = builder.get::<gtk::TextView>("file_new_novel_description");

        let novel_chapters_read = builder.get::<gtk::SpinButton>("file_new_chapters_read");
        let novel_chapters_available = builder.get::<gtk::SpinButton>("file_new_chapters_available");

        let novel_url_entry = builder.get::<gtk::Entry>("file_new_novel_url_entry");
        let novel_image = builder.get::<gtk::Image>("file_new_novel_cover");
        let update_from_url = builder.get::<gtk::CheckButton>("file_new_novel_update");

        // Show cover image
        if let Some(cover) = novel_file_data.cover_data.clone() {
            let fake_file = Cursor::new(cover);
            let cover = Pixbuf::from_read(fake_file).unwrap();
            let cover_img = cover.scale_simple(150, 215, InterpType::Bilinear);
            novel_image.set_from_pixbuf(cover_img.as_ref());
        }

        novel_url_entry.set_text(&novel_file_data.novel_string_id);
        novel_title.set_text(&novel_file_data.novel_title);
        novel_authors.set_text(&novel_file_data.authors);
        novel_genres.set_text(&novel_file_data.genres);
        novel_description
            .buffer()
            .expect("Could not get buffer")
            .set_text(&novel_file_data.description);
        novel_chapters_read.set_value(novel_file_data.chapters.read as f64);
        novel_chapters_available.set_value(novel_file_data.chapters.available as f64);

        self.status_combobox
            .set_active_id(Some(&novel_file_data.status_list_id));

        self.url_combobox.set_active(Some(1));

        update_from_url.set_active(false);
    }
}

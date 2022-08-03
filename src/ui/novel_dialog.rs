use crate::app::novel::{Novel, NovelContentAmount, NovelSettings, NovelStatus, NovelType};
use crate::app::{AppRuntime, NOVEL_UPDATE_COOLDOWN};
use crate::data_dir;
use crate::ui::novel_list::ListStatus;
use crate::utils::gtk::BuilderExtManualCustom;
use crate::utils::nil_str;
use chrono::Local;
use gdk::ModifierType;
use glib::SignalHandlerId;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::WidgetExtManual;
use gtk::prelude::*;
use gtk::{ButtonsType, Dialog, DialogFlags, IconSize, MessageType, ResponseType};
use std::path::Path;
use url::Url;

#[derive(Debug)]
pub struct NovelDialog {
    pub dialog: Dialog,
    // Notebook that shows either the readable info or editable info
    pub notebook: gtk::Notebook,
    // Notebook with the actual content
    pub info_notebook: gtk::Notebook,
    pub edit_notebook: gtk::Notebook,
    pub status_combobox: gtk::ComboBoxText,
    pub novel_status_combobox: gtk::ComboBoxText,
    pub novel_type_combobox: gtk::ComboBoxText,
    pub reading_url_entry: gtk::Entry,
    pub confirm_delete: gtk::MessageDialog,
    pub confirm_update: gtk::MessageDialog,
    pub ok_button: gtk::Button,
    pub update_button: gtk::Button,
    pub update_stack: gtk::Stack,
    link_handler: Option<SignalHandlerId>,
}

impl NovelDialog {
    pub fn new(builder: &gtk::Builder, parent: &gtk::ApplicationWindow) -> NovelDialog {
        let dialog = cascade! {
            builder.get::<gtk::Dialog>("novel_dialog");
            ..set_title(&format!("{} - Eris", &fl!("novel-info")));
            ..set_transient_for(Some(parent));
        };

        // Translate all the things
        builder.button_i18n("novel_btn_ok", &fl!("ok-button"));
        builder.button_i18n("novel_btn_cancel", &fl!("cancel-button"));
        builder.button_i18n("novel_btn_update", &fl!("update-button"));
        builder.button_i18n("novel_btn_edit", &fl!("edit-button"));
        builder.button_i18n("novel_btn_delete", &fl!("delete-button"));

        builder.checkbutton_i18n("rereading_checkbutton", &fl!("novel-rereading"));

        builder.label_i18n("novel_alt_title_label", &fl!("novel-alt-title"));
        builder.label_i18n("novel_details_label", &fl!("novel-details"));
        builder.label_i18n("novel_additional_details_label", &fl!("novel-additional-details"));
        builder.label_i18n("novel_detail_country_label", &(fl!("novel-original-language") + ":"));
        builder.label_i18n("novel_detail_author_label", &(fl!("novel-author") + ":"));
        builder.label_i18n("novel_detail_artist_label", &(fl!("novel-artist") + ":"));
        builder.label_i18n("novel_detail_type_label", &fl!("novel-type"));
        builder.label_i18n("novel_detail_genre_label", &(fl!("novel-genre") + ":"));
        builder.label_i18n("novel_volumes_label", &fl!("volumes"));
        builder.label_i18n("novel_chapters_label", &fl!("chapters"));
        builder.label_i18n("novel_side_stories_label", &fl!("side-stories"));
        builder.label_i18n("novel_description_label", &fl!("novel-description"));
        builder.label_i18n("main_info_tab_label", &fl!("novel-main-info-tab"));
        builder.label_i18n("other_info_tab_label", &fl!("novel-other-info-tab"));
        builder.label_i18n("novel_list_label", &fl!("novel-list"));
        builder.label_i18n("chapters_read_label", &(fl!("novel-chapters-read") + ":"));
        builder.label_i18n("valid_formats_label", &fl!("novel-chapters-format"));
        builder.label_i18n("status_label", &(fl!("status") + ":"));
        builder.label_i18n("score_label", &(fl!("column-score") + ":"));
        builder.label_i18n("year_label", &(fl!("year") + ":"));
        builder.label_i18n("original_publisher_label", &(fl!("original-publishers") + ":"));
        builder.label_i18n("english_publisher_label", &(fl!("english-publishers") + ":"));
        builder.label_i18n("novel_tags_label", &fl!("novel-tags"));
        builder.label_i18n("novel_settings_reading_url_label", &(fl!("novel-reading-url") + ":"));
        builder.label_i18n("novel_settings_reading_file_label", &(fl!("novel-reading-file") + ":"));
        builder.label_i18n(
            "novel_settings_keywords_label",
            &(fl!("novel-rec-keywords") + " ( , ):"),
        );
        builder.label_i18n("novel_settings_notes_label", &(fl!("novel-notes") + ":"));
        builder.label_i18n("novel_settings_label", &fl!("settings"));
        builder.label_i18n("list_and_settings_tab_label", &fl!("novel-list-settings-tab"));
        builder.label_i18n("novel_action_update_label", &fl!("novel-update-label"));
        builder.label_i18n("novel_action_edit_label", &fl!("novel-edit-label"));
        builder.label_i18n("novel_action_delete_label", &fl!("novel-delete-label"));
        builder.label_i18n(
            "novel_setting_last_update_label",
            &(fl!("novel-last-update-label") + ":"),
        );
        builder.label_i18n("actions_tab_label", &fl!("novel-actions-tab"));
        builder.label_i18n("novel_status_edit_label", &(fl!("status") + ":"));
        builder.label_i18n("novel_volumes_edit_label", &(fl!("volumes") + ":"));
        builder.label_i18n("novel_chapters_edit_label", &(fl!("chapters") + ":"));
        builder.label_i18n("novel_side_stories_edit_label", &(fl!("side-stories") + ":"));
        builder.label_i18n("novel_alt_title_edit_label", &fl!("novel-alt-title"));
        builder.label_i18n("novel_details_edit_label", &fl!("novel-details"));
        builder.label_i18n("novel_additional_details_edit_label", &fl!("novel-additional-details"));
        builder.label_i18n(
            "novel_detail_country_edit_label",
            &(fl!("novel-original-language") + ":"),
        );
        builder.label_i18n("novel_detail_author_edit_label", &(fl!("novel-author") + ":"));
        builder.label_i18n("novel_detail_artist_edit_label", &(fl!("novel-artist") + ":"));
        builder.label_i18n("novel_detail_type_edit_label", &(fl!("novel-type") + ":"));
        builder.label_i18n("novel_detail_year_edit_label", &(fl!("year") + ":"));
        builder.label_i18n("novel_detail_genre_edit_label", &(fl!("novel-genre") + ":"));
        builder.label_i18n(
            "novel_detail_original_publisher_edit_label",
            &(fl!("original-publishers") + ":"),
        );
        builder.label_i18n(
            "novel_detail_english_publisher_edit_label",
            &(fl!("english-publishers") + ":"),
        );
        builder.label_i18n("novel_tags_edit_label", &fl!("novel-tags"));

        builder.label_i18n("novel_description_edit_label", &fl!("novel-description"));
        builder.label_i18n("novel_source_edit_label", &(fl!("novel-source") + ":"));
        builder.label_i18n("main_info_tab_edit_label", &fl!("novel-main-info-tab"));
        builder.label_i18n("other_info_tab_edit_label", &fl!("novel-other-info-tab"));

        let notebook = builder.get::<gtk::Notebook>("novel_dialog_notebook");
        let info_notebook = builder.get::<gtk::Notebook>("novel_info_notebook");
        let edit_notebook = builder.get::<gtk::Notebook>("novel_edit_notebook");
        let list_status_comboboxtext = builder.get::<gtk::ComboBoxText>("list_status_comboboxtext");
        let novel_status_comboboxtext = builder.get::<gtk::ComboBoxText>("novel_setting_status_edit");
        let novel_type_combobox = builder.get::<gtk::ComboBoxText>("novel_detail_type_edit");
        let ok_button = builder.get::<gtk::Button>("novel_btn_ok");
        let reading_url_entry = builder.get::<gtk::Entry>("setting_url_entry");
        let update_button = builder.get::<gtk::Button>("novel_btn_update");

        let confirm_delete = cascade! {
            gtk::MessageDialog::new(
                Some(&dialog),
                DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Question,
                ButtonsType::None,
                &fl!("confirm-delete-text")
            );
            ..add_button(&fl!("delete-button"), ResponseType::Ok);
            ..add_button(&fl!("cancel-button"), ResponseType::Cancel);
            ..set_title(&fl!("are-you-sure"));
            ..connect_delete_event(move |dialog, _| { dialog.hide_on_delete() });
        };

        let confirm_update = cascade! {
            gtk::MessageDialog::new(
                Some(&dialog),
                DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Question,
                ButtonsType::None,
                &fl!("confirm-update-text")
            );
            ..add_button(&fl!("update-button"), ResponseType::Ok);
            ..add_button(&fl!("cancel-button"), ResponseType::Cancel);
            ..set_title(&fl!("are-you-sure"));
        };

        let update_stack = builder.get::<gtk::Stack>("update_novel_stack");

        NovelDialog {
            dialog,
            notebook,
            info_notebook,
            edit_notebook,
            status_combobox: list_status_comboboxtext,
            novel_status_combobox: novel_status_comboboxtext,
            novel_type_combobox,
            reading_url_entry,
            confirm_delete,
            confirm_update,
            ok_button,
            update_button,
            update_stack,
            link_handler: None,
        }
    }

    pub fn connect(&self, builder: &gtk::Builder, app_runtime: AppRuntime) {
        let list_status_comboboxtext = self.status_combobox.clone();
        let vcp_read = builder.get::<gtk::Entry>("vcp_read");
        let score_comboboxtext = builder.get::<gtk::ComboBoxText>("score_comboboxtext");
        let rereading_checkbutton = builder.get::<gtk::CheckButton>("rereading_checkbutton");
        let setting_url_entry = builder.get::<gtk::Entry>("setting_url_entry");
        let setting_file = builder.get::<gtk::FileChooserButton>("setting_file");
        let window_titles_entry = builder.get::<gtk::Entry>("window_titles_entry");
        let notes_textview = builder.get::<gtk::TextView>("notes_textview");

        let novel_btn_edit = builder.get::<gtk::Button>("novel_btn_edit");
        let novel_btn_delete = builder.get::<gtk::Button>("novel_btn_delete");

        novel_btn_edit.connect_clicked(glib::clone!(@strong app_runtime => move |_| {
            app_runtime.update_state_with(|state| {
                state.ui.show_novel_dialog_edit()
            });
        }));

        novel_btn_delete.connect_clicked(glib::clone!(@strong self.dialog as dialog => move |_| {
            dialog.response(ResponseType::Reject);
        }));

        // Hide the element instead of deleting it when the close button is clicked
        self.dialog
            .connect_delete_event(move |dialog, _event| dialog.hide_on_delete());

        // Upon pressing CTRL+S send OK response to the dialog so it will
        // save any changes and close
        // This was ENTER but that interfered other things
        self.dialog.connect_key_press_event(|d, e| {
            let ctrl = e.state() == ModifierType::CONTROL_MASK;
            if let Some(keycode) = e.keycode() {
                // CTRL + S
                if ctrl && keycode == 83 {
                    d.response(ResponseType::Ok);
                }
            }
            gtk::Inhibit(false)
        });

        self.dialog.connect_response(glib::clone!(@strong app_runtime, @strong self.notebook as notebook, @strong self.confirm_delete as confirm_delete => move |dialog, response_type| {
            let list_status = ListStatus::from_combo_box_id(list_status_comboboxtext.active_id().unwrap_or_else(|| "0".into()).as_str());
            let read_amount = NovelContentAmount::from_string(vcp_read.text().to_string());
            let score = score_comboboxtext.active_id().unwrap().to_string();
            let rereading = rereading_checkbutton.is_active();
            let reading_url = if setting_url_entry.text().to_string().is_empty() {
                None
            } else {
                Some(setting_url_entry.text().to_string())
            };
            let novel_keywords = if window_titles_entry.text().is_empty() {
                None
            } else {
                let kw = window_titles_entry
                    .text()
                    .split(',')
                    .map(|t|t.trim().to_string())
                    .collect::<Vec<String>>();
                Some(kw)
            };
            let notes_buffer = notes_textview.buffer().unwrap();
            let notes = notes_buffer
                .text(&notes_buffer.start_iter(), &notes_buffer.end_iter(), false)
                .unwrap_or_else(|| "".into())
                .to_string();
            let filepath = setting_file.filename();
            let is_edit_page_open = notebook.page() == 1;

            let novel_settings = NovelSettings {
                list_status,
                content_read: read_amount,
                notes: Some(notes),
                score,
                rereading,
                reading_url,
                window_titles: novel_keywords,
                file: filepath,
                last_read: Local::now().timestamp(),
            };

            match response_type {
                gtk::ResponseType::Ok => {
                    // Check if the edit page is open
                    // if then just switch back to the
                    // non-edit page.
                    if is_edit_page_open {
                        app_runtime.update_state_with(move |state| {
                            // Save any novel edits
                            state.edit_active_novel();
                        });
                        notebook.set_page(0);
                        return;
                    }

                    // Save any changes
                    app_runtime.update_state_with(move |state| {
                        state.edit_novel_settings(novel_settings);
                    });

                    dialog.hide();
                },
                gtk::ResponseType::Cancel => {
                    // Check if the edit page is open
                    // if then just switch back to the
                    // non-edit page.
                    if is_edit_page_open {
                        notebook.set_page(0);
                        return;
                    }

                    dialog.hide()
                },
                gtk::ResponseType::Reject => {
                    // Show delete confirm popup
                    confirm_delete.show();

                    match confirm_delete.run() {
                        ResponseType::Ok => {
                            confirm_delete.hide();
                            dialog.hide();
                            app_runtime.update_state_with(move |state| {
                                state.delete_novel();
                            })
                        },
                        ResponseType::Cancel => {
                            confirm_delete.hide()
                        },
                        _ => {}
                    }
                }
                _ => ()
            }
        }));

        let settings_btn_file_clear = builder.get::<gtk::Button>("settings_btn_file_clear");
        let setting_file = builder.get::<gtk::FileChooserButton>("setting_file");
        settings_btn_file_clear.connect_clicked(move |_| {
            setting_file.unselect_all();
        });

        let update_stack = self.update_stack.clone();
        let confirm_update = self.confirm_update.clone();
        // Fully override link opening so a new browser window/tab does not open
        self.update_button.connect_clicked(move |_| {
            // Show update confirm popup
            confirm_update.show();

            update_stack.set_visible_child_name("page1");

            match confirm_update.run() {
                ResponseType::Ok => {
                    confirm_update.hide();
                    app_runtime.update_state_with(|state| {
                        state.update_novel_from_slug();
                    });
                }
                ResponseType::Cancel => {
                    confirm_update.close();
                }
                _ => {}
            }
        });

        // Change the stack visibility on confirm update close
        let update_stack = self.update_stack.clone();
        self.confirm_update.connect_delete_event(move |dialog, _| {
            update_stack.set_visible_child_name("page0");

            dialog.hide_on_delete()
        });
    }

    pub fn update(&mut self, builder: &gtk::Builder, novel: &Novel) {
        let novel_title_label = builder.get::<gtk::Label>("novel_title_label");
        let image = builder.get::<gtk::Image>("novel_image");
        let novel_alt_title_text = builder.get::<gtk::TextView>("novel_alt_title_text");

        let novel_detail_author_value = builder.get::<gtk::Label>("novel_detail_author_value_label");
        let novel_detail_artist_value = builder.get::<gtk::Label>("novel_detail_artist_value_label");
        let novel_detail_type_value = builder.get::<gtk::Label>("novel_detail_type_value_label");
        let novel_detail_genre_value = builder.get::<gtk::Label>("novel_detail_genre_value_label");
        let novel_detail_country_value = builder.get::<gtk::Label>("novel_detail_country_value_label");
        let novel_tags_text = builder.get::<gtk::TextView>("novel_tags_text");
        let novel_description_text = builder.get::<gtk::TextView>("novel_description_text");
        let novel_source_slug_label = builder.get::<gtk::Label>("novel_source_slug_label");

        let year = builder.get::<gtk::Label>("year_value_label");
        let original_publisher_value = builder.get::<gtk::Label>("original_publisher_value_label");
        let english_publisher_value = builder.get::<gtk::Label>("english_publisher_value_label");

        let novel_volumes_label = builder.get::<gtk::Label>("novel_volumes_label");
        let novel_chapters_label = builder.get::<gtk::Label>("novel_chapters_label");
        let novel_side_stories_label = builder.get::<gtk::Label>("novel_side_stories_label");

        let novel_setting_sides = builder.get::<gtk::Label>("novel_setting_side_stories");
        let novel_setting_chapters = builder.get::<gtk::Label>("novel_setting_chapters");
        let novel_setting_volumes = builder.get::<gtk::Label>("novel_setting_volumes");
        let novel_setting_status = builder.get::<gtk::Label>("novel_setting_status");
        let novel_setting_last_updated = builder.get::<gtk::Label>("novel_setting_last_updated");
        let novel_setting_translated = builder.get::<gtk::Label>("novel_setting_translated");

        let list_status_comboboxtext = builder.get::<gtk::ComboBoxText>("list_status_comboboxtext");
        let vcp_read = builder.get::<gtk::Entry>("vcp_read");
        let score_comboboxtext = builder.get::<gtk::ComboBoxText>("score_comboboxtext");
        let rereading_checkbutton = builder.get::<gtk::CheckButton>("rereading_checkbutton");

        let setting_url_entry = builder.get::<gtk::Entry>("setting_url_entry");
        let setting_file = builder.get::<gtk::FileChooserButton>("setting_file");
        let window_titles_entry = builder.get::<gtk::Entry>("window_titles_entry");
        let notes_textview = builder.get::<gtk::TextView>("notes_textview");

        // Disconnect link handler if one exists
        if let Some(link_handler) = self.link_handler.take() {
            novel_source_slug_label.disconnect(link_handler);
        }

        if let Some(slug) = &novel.slug {
            novel_source_slug_label.set_markup(&format!("<a href=\"{}\">{}</a>", slug, slug));

            let novel_clone = novel.clone();
            let handler = novel_source_slug_label.connect_activate_link(move |_, _| {
                novel_clone.open_slug();

                gtk::Inhibit(true)
            });
            self.link_handler = Some(handler);
        } else {
            novel_source_slug_label.set_markup("");
        }

        novel_title_label.set_label(&novel.title);

        image.set_from_icon_name(Some("gtk-missing-image"), IconSize::Dialog);
        if let Some(image_path) = novel.image.first() {
            let full_path = &data_dir(image_path);
            // If the image exists then try to render it
            if full_path.exists() {
                let pb = Pixbuf::from_file_at_scale(full_path, 150, 215, false).expect("Cannot get Pixbuf");

                image.set_from_pixbuf(Some(&pb));
            }
        }

        if let Some(alt_title) = &novel.alternative_titles {
            novel_alt_title_text.set_height_request(20 * alt_title.len() as i32);
            novel_alt_title_text
                .buffer()
                .expect("Cannot get buffer")
                .set_text(&nil_str(&alt_title.join("\n")));
        } else {
            novel_alt_title_text.set_height_request(18);
            novel_alt_title_text.buffer().expect("Cannot get buffer").set_text("-");
        }

        if novel.content.side_stories > 0 {
            novel_setting_sides.set_text(&novel.content.side_stories.to_string());
            novel_setting_sides.set_visible(true);
            novel_side_stories_label.set_visible(true);
        } else {
            novel_setting_sides.set_visible(false);
            novel_side_stories_label.set_visible(false);
        }

        if novel.content.chapters > 0.0 {
            novel_setting_chapters.set_text(&novel.content.chapters.to_string());
            novel_setting_chapters.set_visible(true);
            novel_chapters_label.set_visible(true);
        } else {
            novel_setting_chapters.set_visible(false);
            novel_chapters_label.set_visible(false);
        }

        if novel.content.volumes > 0 {
            novel_setting_volumes.set_text(&novel.content.volumes.to_string());
            novel_setting_volumes.set_visible(true);
            novel_volumes_label.set_visible(true);
        } else {
            novel_setting_volumes.set_visible(false);
            novel_volumes_label.set_visible(false);
        }

        novel_setting_status.set_text(&novel.status.to_string());
        novel_setting_translated.set_text("");
        if novel.status == NovelStatus::Completed {
            if let Some(translated) = novel.translated() {
                novel_setting_translated.set_text(&translated);
            }
        }

        novel_setting_last_updated.set_text(&novel.last_scrape_string());

        original_publisher_value.set_text(&nil_str(&novel.original_publisher.join("\n")));
        english_publisher_value.set_text(&nil_str(&novel.english_publisher.join("\n")));
        year.set_label(&nil_str(&nil_str(&novel.year.to_string())));

        // Disable the novel update link button if the previous
        // update was done under 1 hour ago and the slug is not supported
        if novel.last_scrape + NOVEL_UPDATE_COOLDOWN > Local::now().timestamp() {
            self.update_button.set_sensitive(false);
        } else {
            self.update_button.set_sensitive(true);
        }

        // Disable the update button if the slug is not supported
        // since it wouldn't do anything anyways
        if !novel.is_slug_supported() {
            self.update_button.set_sensitive(false);
        }

        vcp_read.set_text(&novel.settings.content_read.to_string(false));
        rereading_checkbutton.set_active(novel.settings.rereading);
        score_comboboxtext.set_active_id(Some(&novel.settings.score));
        list_status_comboboxtext.set_active_id(Some(novel.settings.list_status.combo_box_id()));

        if let Some(reading_url) = &novel.settings.reading_url {
            setting_url_entry.set_text(reading_url);
        } else {
            setting_url_entry.set_text("");
        }

        if let Some(window_titles) = &novel.settings.window_titles {
            window_titles_entry.set_text(&window_titles.join(", "));
        } else {
            window_titles_entry.set_text("");
        }

        if let Some(notes) = &novel.settings.notes {
            notes_textview.buffer().expect("Could not get buffer").set_text(notes);
        } else {
            notes_textview.buffer().expect("Could not get buffer").set_text("");
        }

        if let Some(file) = &novel.settings.file {
            setting_file.set_filename(file);
        } else {
            setting_file.unselect_all();
        }

        let novel_type_lang = format!("{} - {}", novel.novel_type.to_string(), novel.orig_lang());

        novel_detail_author_value.set_text(&nil_str(&novel.authors()));
        novel_detail_artist_value.set_text(&nil_str(&novel.artists()));
        novel_detail_type_value.set_text(&novel_type_lang);
        novel_detail_genre_value.set_text(&nil_str(&novel.genres()));
        novel_detail_country_value.set_text(&novel.original_language);

        if novel.tags.is_empty() {
            novel_tags_text
                .buffer()
                .expect("Could not get buffer")
                .set_text(&fl!("no-tags"));
        } else {
            novel_tags_text
                .buffer()
                .expect("Could not get buffer")
                .set_text(&novel.tags());
        }

        if let Some(description) = &novel.description {
            novel_description_text
                .buffer()
                .expect("Could not get buffer")
                .set_text(description);
        } else {
            novel_description_text
                .buffer()
                .expect("Could not get buffer")
                .set_text(&fl!("no-description"));
        }

        self.ok_button.set_label("Ok");
    }

    pub fn update_edit(&self, builder: &gtk::Builder, novel: &Novel) {
        let novel_title_edit = builder.get::<gtk::Entry>("novel_title_edit");
        let image = builder.get::<gtk::Image>("novel_image_edit");
        let novel_alt_title_edit = builder.get::<gtk::TextView>("novel_alt_title_edit");
        let novel_detail_country_edit = builder.get::<gtk::Entry>("novel_detail_country_edit");
        let novel_detail_author_edit = builder.get::<gtk::Entry>("novel_detail_author_edit");
        let novel_detail_artist_edit = builder.get::<gtk::Entry>("novel_detail_artist_edit");
        let novel_detail_genre_edit = builder.get::<gtk::Entry>("novel_detail_genre_edit");
        let novel_detail_original_publisher_edit = builder.get::<gtk::Entry>("novel_detail_original_publisher_edit");
        let novel_detail_english_publisher_edit = builder.get::<gtk::Entry>("novel_detail_english_publisher_edit");
        let novel_tags_edit = builder.get::<gtk::TextView>("novel_tags_edit");
        let novel_description_edit = builder.get::<gtk::TextView>("novel_description_edit");
        let novel_slug_edit = builder.get::<gtk::Entry>("novel_slug_edit");

        let novel_setting_side_stories_edit = builder.get::<gtk::Entry>("novel_setting_side_stories_edit");
        let novel_setting_chapters_edit = builder.get::<gtk::Entry>("novel_setting_chapters_edit");
        let novel_setting_volumes_edit = builder.get::<gtk::Entry>("novel_setting_volumes_edit");
        let novel_setting_year_edit = builder.get::<gtk::Entry>("novel_setting_year_edit");

        self.novel_status_combobox
            .set_active_id(Some(&novel.status.to_i32().to_string()));
        self.novel_type_combobox
            .set_active_id(Some(&novel.novel_type.to_i32().to_string()));

        novel_setting_side_stories_edit.set_text(&novel.content.side_stories.to_string());
        novel_setting_chapters_edit.set_text(&novel.content.chapters.to_string());
        novel_setting_volumes_edit.set_text(&novel.content.volumes.to_string());
        novel_setting_year_edit.set_text(&novel.year.to_string());

        novel_title_edit.set_text(&novel.title);
        novel_detail_country_edit.set_text(&novel.original_language);
        novel_detail_author_edit.set_text(&novel.authors());
        novel_detail_artist_edit.set_text(&novel.artists());
        novel_detail_genre_edit.set_text(&novel.genres());
        novel_detail_original_publisher_edit.set_text(&novel.original_publishers());
        novel_detail_english_publisher_edit.set_text(&novel.english_publishers());

        image.set_from_icon_name(Some("gtk-missing-image"), IconSize::Dialog);
        if let Some(image_path) = novel.image.first() {
            // Make the image path correct
            let working_image_path = data_dir(image_path);
            // If the image exists then try to render it
            if Path::new(&working_image_path).exists() {
                let pb = Pixbuf::from_file_at_scale(Path::new(&working_image_path), 150, 215, false)
                    .expect("Cannot get Pixbuf");

                image.set_from_pixbuf(Some(&pb));
            }
        }

        if let Some(alt_titles) = &novel.alternative_titles {
            novel_alt_title_edit
                .buffer()
                .expect("Could not get buffer")
                .set_text(&alt_titles.join("\n"));
        } else {
            novel_alt_title_edit
                .buffer()
                .expect("Could not get buffer")
                .set_text("");
        }

        if novel.tags.is_empty() {
            novel_tags_edit
                .buffer()
                .expect("Could not get buffer")
                .set_text(&fl!("no-tags"));
        } else {
            novel_tags_edit
                .buffer()
                .expect("Could not get buffer")
                .set_text(&novel.tags());
        }

        if let Some(description) = &novel.description {
            novel_description_edit
                .buffer()
                .expect("Could not get buffer")
                .set_text(description);
        } else {
            novel_description_edit
                .buffer()
                .expect("Could not get buffer")
                .set_text("");
        }

        if let Some(uri) = &novel.slug {
            novel_slug_edit.set_text(uri);
        } else {
            novel_slug_edit.set_text("");
        }

        self.ok_button.set_label("Save");
    }

    pub fn update_novel_from_edit(&self, builder: &gtk::Builder, novel: &Novel) -> Novel {
        let novel_title_edit = builder.get::<gtk::Entry>("novel_title_edit");
        let novel_alt_title_edit = builder.get::<gtk::TextView>("novel_alt_title_edit");
        let novel_detail_country_edit = builder.get::<gtk::Entry>("novel_detail_country_edit");
        let novel_detail_author_edit = builder.get::<gtk::Entry>("novel_detail_author_edit");
        let novel_detail_artist_edit = builder.get::<gtk::Entry>("novel_detail_artist_edit");
        let novel_detail_genre_edit = builder.get::<gtk::Entry>("novel_detail_genre_edit");
        let novel_detail_original_publisher_edit = builder.get::<gtk::Entry>("novel_detail_original_publisher_edit");
        let novel_detail_english_publisher_edit = builder.get::<gtk::Entry>("novel_detail_english_publisher_edit");
        let novel_tags_edit = builder.get::<gtk::TextView>("novel_tags_edit");
        let novel_description_edit = builder.get::<gtk::TextView>("novel_description_edit");
        let novel_slug_edit = builder.get::<gtk::Entry>("novel_slug_edit");

        let novel_setting_side_stories_edit = builder.get::<gtk::Entry>("novel_setting_side_stories_edit");
        let novel_setting_chapters_edit = builder.get::<gtk::Entry>("novel_setting_chapters_edit");
        let novel_setting_volumes_edit = builder.get::<gtk::Entry>("novel_setting_volumes_edit");
        let novel_setting_year_edit = builder.get::<gtk::Entry>("novel_setting_year_edit");

        let alt_titles_buffer = novel_alt_title_edit.buffer().expect("Cannot get buffer");
        let alt_titles = alt_titles_buffer
            .text(&alt_titles_buffer.start_iter(), &alt_titles_buffer.end_iter(), false)
            .unwrap()
            .to_string()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect::<Vec<String>>();

        let alternative_titles = if !alt_titles.is_empty() { Some(alt_titles) } else { None };

        let authors = novel_detail_author_edit
            .text()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let artists = novel_detail_artist_edit
            .text()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let genres = novel_detail_genre_edit
            .text()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let original_publisher = novel_detail_original_publisher_edit
            .text()
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let english_publisher = novel_detail_english_publisher_edit
            .text()
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let tags_buffer = novel_tags_edit.buffer().expect("Cannot get buffer");
        let got_tags: Option<String> = tags_buffer
            .text(&tags_buffer.start_iter(), &tags_buffer.end_iter(), false)
            .and_then(|v| v.parse().ok())
            .filter(|s: &String| !s.is_empty());
        let tags: Vec<String> = if let Some(tag_string) = got_tags {
            // Tag vector from strings
            let mut tag_vec = tag_string
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();
            // Sort tags
            tag_vec.sort();
            // Return sorted tags
            tag_vec
        } else {
            vec![]
        };

        let description_buffer = novel_description_edit.buffer().expect("Cannot get buffer");
        let description = description_buffer
            .text(&description_buffer.start_iter(), &description_buffer.end_iter(), false)
            .and_then(|v| v.parse().ok())
            .filter(|s: &String| !s.is_empty());

        let slug = novel_slug_edit.text().to_string();
        let source = if let Ok(url) = Url::parse(&slug) {
            Some(url.domain().unwrap_or("").to_string())
        } else {
            None
        };

        let new_status = NovelStatus::from_i32(
            self.novel_status_combobox
                .active_id()
                .unwrap()
                .to_string()
                .parse::<i32>()
                .unwrap(),
        );

        Novel {
            title: novel_title_edit.text().to_string(),
            alternative_titles,
            author: authors,
            artist: artists,
            novel_type: NovelType::from_i32(
                self.novel_type_combobox
                    .active_id()
                    .unwrap()
                    .to_string()
                    .parse::<i32>()
                    .unwrap(),
            ),
            genre: genres,
            tags,
            year: novel_setting_year_edit.text().to_string().parse::<i32>().unwrap(),
            description,
            source,
            original_language: novel_detail_country_edit.text().to_string(),
            original_publisher,
            english_publisher,
            slug: Some(slug),
            content: NovelContentAmount {
                volumes: novel_setting_volumes_edit
                    .text()
                    .to_string()
                    .parse::<i32>()
                    .unwrap_or(novel.content.volumes),
                chapters: novel_setting_chapters_edit
                    .text()
                    .to_string()
                    .parse::<f32>()
                    .unwrap_or(novel.content.chapters),
                side_stories: novel_setting_side_stories_edit
                    .text()
                    .to_string()
                    .parse::<i32>()
                    .unwrap_or(novel.content.side_stories),
            },
            status: new_status,
            ..novel.clone()
        }
    }
}

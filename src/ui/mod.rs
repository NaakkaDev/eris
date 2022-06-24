mod about_dialog;
mod exporter;
mod file_new_dialog;
mod filter;
pub(crate) mod history;
pub(crate) mod new_dialog;
mod notifcation_dialog;
mod novel_dialog;
pub(crate) mod novel_list;
mod reading_now;
mod settings_dialog;

pub use self::novel_list::NovelList;
use gdk::gdk_pixbuf::Pixbuf;
use glib::SignalHandlerId;
use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

use crate::app::localize::available_languages;
use crate::app::novel::{Novel, NovelStatus, NovelType};
use crate::app::settings::{ChapterReadPreference, Settings};
use crate::app::AppRuntime;
use crate::ui::file_new_dialog::FileNewNovelDialog;
use crate::ui::filter::FilterList;
use crate::ui::history::HistoryList;
use crate::ui::new_dialog::NewNovelDialog;
use crate::ui::novel_dialog::NovelDialog;
use crate::ui::novel_list::ListStatus;
use crate::ui::settings_dialog::SettingsDialog;
use crate::utils::gtk::BuilderExtManualCustom;
use crate::utils::Resources;
use crate::UPDATE_LINK;
use gtk::prelude::*;

pub struct UI {
    pub builder: gtk::Builder,
    pub gtk_app: gtk::Application,
    pub main_window: gtk::ApplicationWindow,

    pub lists: NovelList,
    pub filter: FilterList,
    pub history: HistoryList,
    pub main_notebook: gtk::Notebook,
    pub list_notebook: gtk::Notebook,
    pub main_tabs: Vec<gtk::Box>,
    pub reading_notebook: gtk::Notebook,

    pub novel_dialog: NovelDialog,
    pub new_dialog: NewNovelDialog,
    pub file_new_dialog: FileNewNovelDialog,
    pub settings_dialog: SettingsDialog,

    pub url_list: Vec<String>,
    link_handler: Option<SignalHandlerId>,
}

impl UI {
    pub fn new(gtk_app: gtk::Application) -> UI {
        let builder = gtk::Builder::from_resources("ui/main_window.ui");
        builder.add_from_resources("ui/novel_dialog.ui");
        builder.add_from_resources("ui/new_dialog.ui");
        builder.add_from_resources("ui/settings_dialog.ui");
        builder.add_from_resources("ui/file_new_dialog.ui");

        let main_window = builder.get::<gtk::ApplicationWindow>("main_window");

        main_window.set_application(Some(&gtk_app));
        main_window.set_title("Eris");

        let resource = Resources::get("icons/eris.ico").unwrap().data;
        let icon_pix = Pixbuf::from_read(Cursor::new(resource)).expect("Cannot load pixbuf from resource.");
        main_window.set_icon(Some(&icon_pix));

        // Translate all the things
        builder.label_i18n("select_reading_label", &fl!("currently-reading"));
        builder.label_i18n("select_novel_list_label", &fl!("novel-list"));
        builder.label_i18n("select_history_label", &fl!("history"));

        builder.label_i18n("reading_reading_label", &(fl!("reading") + ".."));
        builder.label_i18n("reading_chapter_label", &(fl!("chapter") + ":"));
        builder.label_i18n("reading_volume_label", &(fl!("volume") + ":"));
        builder.label_i18n("reading_side_story_label", &(fl!("side-story") + ":"));
        builder.label_i18n("reading_alt_title_label", &fl!("novel-alt-title"));
        builder.label_i18n("reading_details_label", &fl!("novel-details"));
        builder.label_i18n("reading_author_label", &(fl!("novel-author") + ":"));
        builder.label_i18n("reading_artist_label", &(fl!("novel-artist") + ":"));
        builder.label_i18n("reading_genre_label", &(fl!("novel-genre") + ":"));
        builder.label_i18n("reading_description_label", &fl!("novel-description"));
        builder.label_i18n("reading_source_label", &(fl!("novel-source") + ":"));
        builder.label_i18n("not_found_label", &fl!("not-found"));
        builder.label_i18n("not_found_suggestion_label", &fl!("not-found-suggestion"));
        builder.label_i18n("previously_read_label", &(fl!("previously-read") + ":"));
        builder.label_i18n("currently_reading_label", &fl!("currently-reading"));
        builder.label_i18n("menu_update_label", &fl!("menu-update"));
        builder.label_i18n("label_load_history", &fl!("load-label"));

        builder.button_i18n("btn_add", &fl!("add-button"));
        builder.button_i18n("btn_continue_reading", &fl!("continue-reading-button"));
        builder.button_i18n("btn_reading_type", &fl!("reading-type-button"));
        builder.button_i18n("btn_load_history", &fl!("load-history-button"));

        builder.menu_item_i18n("menu_file", &fl!("menu-file"));
        builder.menu_item_i18n("menu_tools", &fl!("menu-tools"));
        builder.menu_item_i18n("menu_view", &fl!("menu-view"));
        builder.menu_item_i18n("menu_help", &fl!("menu-help"));
        builder.menu_item_i18n("menu_new", &fl!("menu-new"));
        builder.menu_item_i18n("menu_save", &fl!("menu-save"));
        builder.menu_item_i18n("menu_quit", &fl!("menu-quit"));
        builder.menu_item_i18n("menu_settings", &fl!("settings"));
        builder.menu_item_i18n("show_reading_checkmenuitem", &fl!("menu-show-reading"));
        builder.menu_item_i18n("show_list_checkmenuitem", &fl!("menu-show-list"));
        builder.menu_item_i18n("show_history_checkmenuitem", &fl!("menu-show-history"));
        builder.menu_item_i18n("menu_about", &fl!("menu-about"));
        builder.menu_checkitem_i18n("toggle_novel_recognition", &fl!("novel-recognition"));
        builder.menu_checkitem_i18n("show_sidebar_checkmenuitem", &fl!("menu-show-sidebar"));

        builder
            .get::<gtk::Button>("btn_new")
            .set_tooltip_text(Some(&fl!("add-novel")));
        builder
            .get::<gtk::Button>("btn_settings")
            .set_tooltip_text(Some(&fl!("open-settings")));

        let main_notebook = builder.get::<gtk::Notebook>("main_notebook");
        let reading_notebook = builder.get::<gtk::Notebook>("reading_notebook");
        let novel_list_listboxrow = builder.get::<gtk::ListBoxRow>("novel_list_listboxrow");
        // Notebook that has all the lists
        let list_notebook = builder.get::<gtk::Notebook>("novel_list_notebook");

        main_notebook.set_current_page(Some(1));
        novel_list_listboxrow.activate();

        let lists = NovelList::new(&builder);
        let filter = FilterList::new(&builder);
        let history = HistoryList::new(&builder);
        let novel_dialog = NovelDialog::new(&builder, &main_window);
        let new_dialog = NewNovelDialog::new(&builder, &main_window);
        let file_new_dialog = FileNewNovelDialog::new(&builder, &main_window);
        let settings_dialog = SettingsDialog::new(&builder, &main_window);

        let url_list = vec![
            fl!("other"),
            "https://www.novelupdates.com/series/".to_string(),
            "https://www.royalroad.com/fiction/".to_string(),
            "https://www.scribblehub.com/series/".to_string(),
            "https://www.webnovel.com/book/".to_string(),
        ];

        let new_icon = Resources::get("icons/new_icon.png").unwrap().data;
        let new_pix = Pixbuf::from_read(Cursor::new(new_icon)).expect("Cannot load pixbuf from resource.");
        let btn_new_img = builder.get::<gtk::Image>("btn_new_img");
        btn_new_img.set_pixbuf(Some(&new_pix));

        let settings_icon = Resources::get("icons/settings_icon.png").unwrap().data;
        let settings_pix = Pixbuf::from_read(Cursor::new(settings_icon)).expect("Cannot load pixbuf from resource.");
        let btn_settings_img = builder.get::<gtk::Image>("btn_settings_img");
        btn_settings_img.set_pixbuf(Some(&settings_pix));

        UI {
            builder,
            gtk_app,
            main_window,
            lists,
            filter,
            history,
            main_notebook,
            list_notebook,
            main_tabs: vec![],
            reading_notebook,
            novel_dialog,
            new_dialog,
            file_new_dialog,
            settings_dialog,
            url_list,
            link_handler: None,
        }
    }

    pub fn init(&self) {
        for combobox in self.settings_dialog.action_comboboxes.values() {
            self.populate_combobox(combobox, &self.settings_dialog.action_list);
        }
        self.populate_combobox(&self.settings_dialog.preference_combobox, &ChapterReadPreference::vec());
        self.populate_combobox(&self.novel_dialog.status_combobox, &ListStatus::vec());
        self.populate_combobox(&self.novel_dialog.novel_status_combobox, &NovelStatus::vec());
        self.populate_combobox(&self.novel_dialog.novel_type_combobox, &NovelType::vec());
        self.populate_combobox(&self.new_dialog.status_combobox, &ListStatus::vec());
        self.populate_combobox(&self.file_new_dialog.status_combobox, &ListStatus::vec());
        self.populate_combobox(&self.file_new_dialog.url_combobox, &self.url_list);
        self.populate_combobox(&self.new_dialog.url_combobox, &self.url_list);
        self.populate_combobox(
            &self.settings_dialog.novel_info_tabs_combobox,
            &[
                "Main information",
                "Other information",
                "My list and settings",
                "Actions",
            ],
        );
        self.populate_language_combobox(&self.settings_dialog.language_combobox, &available_languages());

        debug!("UI init doned");
    }

    pub fn connect(&self, app_runtime: AppRuntime) {
        let main_notebook = self.builder.get::<gtk::Notebook>("main_notebook");
        let view_selection_listbox = self.builder.get::<gtk::ListBox>("view_selection_listbox");
        let btn_continue_reading = self.builder.get::<gtk::Button>("btn_continue_reading");
        let btn_reading_type = self.builder.get::<gtk::Button>("btn_reading_type");
        let btn_reading_info = self.builder.get::<gtk::Button>("btn_reading_info");
        let btn_load_history = self.builder.get::<gtk::Button>("btn_load_history");

        main_notebook.connect_switch_page(glib::clone!(@strong app_runtime => move |_, _, page| {
            app_runtime.update_state_with(move |state| {
                state.switched_main_notebook_page(page);
            });
        }));

        view_selection_listbox.connect_row_activated(move |_, row| {
            main_notebook.set_current_page(Some(row.index() as u32));
        });

        btn_continue_reading.connect_button_release_event(glib::clone!(@strong app_runtime => move |_, _| {
            app_runtime.update_state_with(move |state| {
                if let Some(last_read) = state.history.clone().read().find_last_read() {
                    if let Some(novel) = state.get_by_id(last_read.novel_id) {
                        state.read_novel(novel);
                    }
                }
            });

            gtk::Inhibit(false)
        }));

        btn_reading_type.connect_clicked(glib::clone!(@strong app_runtime => move |btn| {
            btn.set_visible(false);
            app_runtime.update_state_with(|state| {
                let novel_reading = state.currently_reading.novel.read().clone();
                if let Some(novel) = novel_reading {
                    let updated_novel = state.update_novel_status(novel, NovelStatus::Completed);
                    state.ui.novel_dialog.update_edit(&state.ui.builder, &updated_novel);
                    state.ui.lists.active_novel = Some(updated_novel.clone());
                    state.ui.update_reading_now(&Some(updated_novel));
                }

                state.ui.close_reading_popover();
            })
        }));

        btn_reading_info.connect_clicked(glib::clone!(@strong app_runtime => move |_btn| {
            app_runtime.update_state_with(|state| {
                if let Some(potentially_old_novel) = state.currently_reading.novel.read().clone() {
                    if let Some(novel) = state.get_by_id(potentially_old_novel.id) {
                        state.ui.lists.active_list = ListStatus::from_i32(novel.settings.list_status.to_i32());
                        state.ui.lists.active_iter = state.ui.lists.find_iter(&novel);
                        state.ui.lists.active_novel = Some(novel.clone());
                        state.ui.show_novel_dialog(&novel, &state.settings.read());
                    }
                }

                state.ui.close_reading_popover();
            })
        }));

        // Load all history on button click
        let label_load_history = self.builder.get::<gtk::Label>("label_load_history");
        btn_load_history.connect_clicked(glib::clone!(@strong app_runtime => move |btn| {
            let label_load_history = label_load_history.clone();
            app_runtime.update_state_with(move |state| {
                let load_start = Instant::now();
                // Clear the history list
                state.ui.history.list_clear();
                // Now load all the entries into it
                state.ui.history.populate_columns(&state.history.read().items);
                // Tell user the cool stuff
                label_load_history.set_text(&format!(
                    "Loaded {:?} history entries in {:?}",
                    &state.history.read().items.len(),
                    load_start.elapsed()
                ));
            });
            // Hide the button as it is not needed anymore
            btn.set_visible(false);
        }));

        self.main_window
            .connect_destroy(glib::clone!(@strong app_runtime => move |_| {
                app_runtime.update_state_with(move |state| {
                    state.ui.gtk_app.quit();
                });
            }));

        self.main_window
            .connect_window_state_event(glib::clone!(@strong app_runtime => move |_w, e| {
                // Left the iconified state (deiconified)
                if e.changed_mask() == gdk::WindowState::ICONIFIED {
                    app_runtime.update_state_with(move |state| {
                        if let Some(window_state) = state.window_state.read().clone() {
                            // Maximize the window if it should be
                            if window_state.is_maximized {
                                state.ui.main_window.maximize();
                            }
                        }
                    });
                }

                gtk::Inhibit(false)
            }));

        // Update the `selected_page` value
        self.list_notebook
            .connect_page_notify(glib::clone!(@strong app_runtime => move |notebook| {
                let page = notebook.page();

                // Ignore pages above 4, so the filter page.
                if page < 5 {
                    app_runtime.update_state_with(move |state| {
                        state.ui.filter.selected_page = page;
                        state.ui.lists.active_page = page;
                    });
                }
            }));
    }

    /// Populate `gtk:ComboBoxText` from a list a `S`s, where `S` can be `String`, `str` or their refs.
    fn populate_combobox<S: AsRef<str>>(&self, combobox: &gtk::ComboBoxText, values: &[S]) {
        for (key, value) in values.iter().enumerate() {
            combobox.append(Some(key.to_string().as_str()), value.as_ref());
        }
    }

    fn populate_language_combobox(&self, combobox: &gtk::ComboBoxText, hashmap: &HashMap<String, String>) {
        let mut as_vec: Vec<_> = hashmap.iter().collect();
        as_vec.sort_by_key(|a| a.1);

        combobox.append(Some("none"), &fl!("system-default"));
        for (key, value) in as_vec {
            combobox.append(Some(key), value);
        }
    }

    pub fn init_menu(&self, settings: &Settings) {
        let view_selection_listbox = self.builder.get::<gtk::ListBox>("view_selection_listbox");
        let show_sidebar_checkmenuitem = self.builder.get::<gtk::CheckMenuItem>("show_sidebar_checkmenuitem");

        // Triggers the `toggle_sidebar` since that is connected to the activation
        show_sidebar_checkmenuitem.set_active(settings.general.show_sidebar);
        view_selection_listbox.set_visible(settings.general.show_sidebar);

        let toggle_novel_recognition = self.builder.get::<gtk::CheckMenuItem>("toggle_novel_recognition");

        // `toggle_novel_recognition`
        toggle_novel_recognition.set_active(settings.novel_recognition.enable);

        // Set the desired novel dialog notebook page
        self.novel_dialog
            .info_notebook
            .set_page(settings.list.open_info_behavior);
    }

    /// Show reading now notebook page.
    pub fn show_reading_now(&self) {
        let reading_now_listboxrow = self.builder.get::<gtk::ListBoxRow>("reading_now_listboxrow");
        reading_now_listboxrow.activate();

        self.main_notebook.set_current_page(Some(0));
    }

    /// Show novel list notebook page.
    pub fn show_novel_list(&self) {
        let novel_list_listboxrow = self.builder.get::<gtk::ListBoxRow>("novel_list_listboxrow");
        novel_list_listboxrow.activate();

        self.main_notebook.set_current_page(Some(1));
    }

    /// Show novel list notebook page without stealing focus.
    pub fn show_novel_list_no_focus(&self) {
        let novel_list_listboxrow = self.builder.get::<gtk::ListBoxRow>("novel_list_listboxrow");
        // Do not allow focus
        novel_list_listboxrow.set_can_focus(false);
        // Activate this row
        novel_list_listboxrow.activate();
        // Allow focus again
        novel_list_listboxrow.set_can_focus(true);

        self.main_notebook.set_current_page(Some(1));
    }

    /// Show next notebook page of the current notebook..
    pub fn hotkey_next_active_list(&mut self) {
        if self.main_notebook.page() == 1 {
            if self.list_notebook.page() == 4 {
                self.list_notebook.set_page(0);
            } else {
                self.list_notebook.next_page();
            }
        }
    }

    /// Show history notebook page.
    pub fn show_history(&self) {
        let history_listboxrow = self.builder.get::<gtk::ListBoxRow>("history_listboxrow");
        history_listboxrow.activate();

        self.main_notebook.set_current_page(Some(2));
    }

    /// Open novel dialog window.
    pub fn show_novel_dialog(&mut self, novel: &Novel, settings: &Settings) {
        debug!("ui::show_novel_dialog | novel: {:?}", novel);

        if settings.list.always_open_selected_tab {
            self.novel_dialog
                .info_notebook
                .set_page(settings.list.open_info_behavior);
        }

        self.novel_dialog.notebook.set_page(0);
        self.novel_dialog.update(&self.builder, novel);
        self.novel_dialog.dialog.show();
    }

    /// Open novel dialog window with edit notebook page visible.
    pub fn show_novel_dialog_edit(&mut self) {
        if let Some(novel) = self.lists.active_novel.clone() {
            debug!("ui::show_novel_dialog_edit | novel: {:?}", novel);
            self.novel_dialog.notebook.set_page(1);
            self.novel_dialog.update(&self.builder, &novel);
            self.novel_dialog.update_edit(&self.builder, &novel);
            self.novel_dialog.dialog.show();
        }
    }

    /// Open novel dialog window and show reading url entry field with focus.
    pub fn show_novel_dialog_reading_settings(&mut self, novel: &Novel) {
        debug!("ui::show_novel_dialog_reading_settings | novel: {:?}", novel);
        self.novel_dialog.notebook.set_page(0);
        self.novel_dialog.info_notebook.set_page(2);
        self.novel_dialog.update(&self.builder, novel);
        self.novel_dialog.dialog.show();
    }

    /// Open new dialog window.
    pub fn show_new_dialog(&self) {
        self.new_dialog.update(&self.builder);
        self.new_dialog.dialog.show();
    }

    /// Open settings dialog window.
    pub fn show_settings_dialog(&self, settings: Settings) {
        self.settings_dialog.update(&self.builder, &settings);
        self.settings_dialog.dialog.show();
    }

    /// Show or hide the update menu button
    pub fn toggle_update_menu(&self, show: bool) {
        let update_btn = self.builder.get::<gtk::Label>("menu_update_label");

        update_btn.set_markup(&format!("<a href=\"{}\">{}</a>", UPDATE_LINK, &fl!("menu-update")));

        // This seeimgly does nothing but eh
        update_btn.set_visible(show);
    }

    pub fn close_reading_popover(&self) {
        let reading_popover = self.builder.get::<gtk::Popover>("reading_popover");
        reading_popover.hide();
    }
}

use crate::app::settings::{NovelListAction, Settings};
use crate::app::AppRuntime;
use crate::ui::novel_list::Column;
use crate::utils::gtk::BuilderExtManualCustom;
use gtk::prelude::WidgetExtManual;
use gtk::prelude::*;
use gtk::Dialog;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SettingsDialog {
    pub dialog: Dialog,
    pub action_list: Vec<String>,
    pub action_comboboxes: HashMap<&'static str, gtk::ComboBoxText>,
    pub preference_combobox: gtk::ComboBoxText,
    pub language_combobox: gtk::ComboBoxText,
    pub novel_info_tabs_combobox: gtk::ComboBoxText,
}

impl SettingsDialog {
    pub fn new(builder: &gtk::Builder, parent: &gtk::ApplicationWindow) -> SettingsDialog {
        let dialog = cascade! {
            builder.get::<gtk::Dialog>("settings_dialog");
            ..set_title(&format!("{} - Eris", fl!("settings")));
            ..set_modal(true);
            ..set_transient_for(Some(parent));
            ..connect_response(|dialog, response_type| {
                match response_type {
                    gtk::ResponseType::Apply => dialog.hide(),
                    gtk::ResponseType::Cancel => dialog.hide(),
                    _ => ()
                }
            });
        };

        // Translate all the things
        builder.button_i18n("settings_btn_ok", &fl!("ok-button"));
        builder.button_i18n("settings_btn_cancel", &fl!("cancel-button"));
        builder.button_i18n("settings_btn_default", &fl!("default-settings-button"));
        builder.button_i18n("settings_btn_file_clear", &fl!("unset-button"));
        builder.button_i18n("settings_btn_open_dir", &fl!("open-dir-button"));

        builder.label_i18n("application_settings_label", &fl!("settings-app"));
        builder.label_i18n("novel_list_label", &fl!("novel-list"));
        builder.label_i18n("novel_recognition_label", &fl!("novel-recognition"));
        builder.label_i18n("data_label", &fl!("settings-data"));

        builder.label_i18n("settings_app_label", &fl!("settings-app"));
        builder.label_i18n(
            "settings_actions_label",
            &fl!("settings-app-list-actions-title"),
        );
        builder.label_i18n("mouse_actions_1_label", &(fl!("settings-app-mmc") + ":"));
        builder.label_i18n("mouse_actions_2_label", &(fl!("settings-app-mrc") + ":"));
        builder.label_i18n("mouse_actions_3_label", &(fl!("settings-app-mb4") + ":"));
        builder.label_i18n("mouse_actions_4_label", &(fl!("settings-app-mb5") + ":"));
        builder.label_i18n(
            "settings_columns_label",
            &fl!("settings-app-list-columns-title"),
        );
        builder.label_i18n("settings_interface_label", &fl!("settings-app-interface"));
        builder.label_i18n(
            "settings_language_label",
            &(fl!("settings-app-language") + ":"),
        );
        builder.label_i18n(
            "settings_language_hint_label",
            &fl!("settings-app-language-hint"),
        );
        builder.label_i18n("settings_startup_label", &fl!("settings-app-startup-title"));
        builder.label_i18n("settings_system_label", &fl!("settings-app-system-title"));
        builder.label_i18n(
            "settings_window_state_label",
            &(fl!("settings-app-window-state") + ":"),
        );
        builder.label_i18n("settings_reader_label", &(fl!("settings-app-reader") + ":"));
        builder.label_i18n(
            "settings_reader_arguments_label",
            &(fl!("settings-app-arguments") + ":"),
        );
        builder.label_i18n(
            "settings_window_state_text_label",
            &fl!("window-state-text"),
        );

        builder.label_i18n("settings_reg_label", &fl!("novel-recognition"));
        builder.label_i18n("settings_basic_label", &fl!("settings-reg-basic-title"));
        builder.label_i18n(
            "settings_reg_chapter_pref_label",
            &(fl!("settings-reg-chapter-read-pref") + ":"),
        );
        builder.label_i18n(
            "settings_reg_autocomplete_ongoing",
            &(fl!("settings-reg-autocomplete-ongoing") + ":"),
        );
        builder.label_i18n(
            "settings_reg_enable_label",
            &(fl!("settings-reg-enable-feature") + ":"),
        );
        builder.label_i18n(
            "settings_rec_delay_label",
            &(fl!("settings-reg-delay") + ":"),
        );
        builder.label_i18n(
            "settings_rec_delay_info_label",
            &fl!("settings-reg-delay-info"),
        );

        builder.label_i18n("settings_adv_label", &fl!("settings-reg-advanced-title"));
        builder.label_i18n(
            "settings_keywords_label",
            &(fl!("settings-reg-keywords") + ":"),
        );
        builder.label_i18n(
            "settings_ignore_label",
            &(fl!("settings-reg-ignore-keywords") + ":"),
        );
        builder.label_i18n("settings_behavior_label", &fl!("settings-rec-behavior"));
        builder.label_i18n(
            "settings_rec_when_rec_label",
            &(fl!("settings-rec-when") + ":"),
        );
        builder.label_i18n(
            "settings_rec_when_not_rec_label",
            &(fl!("settings-rec-when-not") + ":"),
        );
        builder.label_i18n(
            "settings_list_behavior_label",
            &fl!("settings-rec-behavior"),
        );
        builder.label_i18n(
            "settings_list_behavior_text_label",
            &(fl!("settings-list-behavior-text") + ":"),
        );

        builder.checkbutton_i18n("settings_window_state_enabled", &fl!("enabled"));
        builder.checkbutton_i18n("settings_startup_auto", &fl!("windows-auto-startup"));
        builder.checkbutton_i18n(
            "settings_startup_minimized",
            &fl!("windows-start-minimized"),
        );
        builder.checkbutton_i18n(
            "settings_startup_check_update",
            &fl!("windows-start-check-update"),
        );
        builder.checkbutton_i18n("viscol_status", &fl!("status"));
        builder.checkbutton_i18n("viscol_cco", &fl!("novel-original-language"));
        builder.checkbutton_i18n("viscol_name", &fl!("column-name"));
        builder.checkbutton_i18n("viscol_ch", &fl!("column-chapters-read"));
        builder.checkbutton_i18n("viscol_vol", &fl!("column-volumes-read"));
        builder.checkbutton_i18n("viscol_side", &fl!("column-side-stories-read"));
        builder.checkbutton_i18n("viscol_avail", &fl!("column-availability"));
        builder.checkbutton_i18n("viscol_score", &fl!("column-score"));
        builder.checkbutton_i18n("viscol_last", &fl!("column-last-update"));
        builder.checkbutton_i18n("first_tab_always_checkbox", &fl!("first-tab-always"));
        builder.checkbutton_i18n("novel_recognition_enabled_checkbutton", &fl!("yes"));
        builder.checkbutton_i18n("novel_recognition_autocomplete_ongoing", &fl!("yes"));
        builder.checkbutton_i18n("novel_rec_found_go_to_reading", &fl!("go-to-reading-now"));
        builder.checkbutton_i18n(
            "novel_rec_not_found_go_to_reading",
            &fl!("go-to-reading-now"),
        );

        builder.label_i18n("settings_data_label", &fl!("settings-data"));
        builder.label_i18n("settings_data_dir_label", &fl!("settings-data-dir"));

        let action_list = NovelListAction::vec();

        cascade! {
            builder.get::<gtk::ListBoxRow>("application_settings_listboxrow");
            ..activate();
        };

        let action_comboboxes = hashmap![
            "mouse_2_action" => builder.get::<gtk::ComboBoxText>("mouse_2_action"),
            "mouse_3_action" => builder.get::<gtk::ComboBoxText>("mouse_3_action"),
            "mouse_4_action" => builder.get::<gtk::ComboBoxText>("mouse_4_action"),
            "mouse_5_action" => builder.get::<gtk::ComboBoxText>("mouse_5_action")
        ];

        let preference_combobox =
            builder.get::<gtk::ComboBoxText>("novel_recognition_read_preference_combobox");
        let language_combobox = builder.get::<gtk::ComboBoxText>("language_combobox");
        let novel_info_tabs_combobox =
            builder.get::<gtk::ComboBoxText>("first_tab_behavior_combobox");

        SettingsDialog {
            dialog,
            action_list,
            action_comboboxes,
            preference_combobox,
            language_combobox,
            novel_info_tabs_combobox,
        }
    }

    pub fn connect(&self, builder: &gtk::Builder, app_runtime: AppRuntime) {
        let settings_notebook = builder.get::<gtk::Notebook>("settings_notebook");
        let settings_listbox = builder.get::<gtk::ListBox>("settings_listbox");
        let settings_btn_default = builder.get::<gtk::Button>("settings_btn_default");

        settings_btn_default.connect_clicked(
            glib::clone!(@strong app_runtime, @strong self.dialog as dialog => move |_| {
                dialog.hide();
                app_runtime.update_state_with(move |state| {
                    state.set_default_settings();
                });
            }),
        );

        // Hide the element instead of deleting it when the close button is clicked
        self.dialog
            .connect_delete_event(move |dialog, _event| dialog.hide_on_delete());

        settings_listbox.connect_row_activated(move |_list, row| {
            settings_notebook.set_current_page(Some(row.index() as u32));
        });

        self.dialog.connect_response(
            glib::clone!(@strong app_runtime => move |dialog, response_type| {
                match response_type {
                    gtk::ResponseType::Ok => {
                        dialog.hide();

                        app_runtime.update_state_with(move |state| {
                            state.update_settings();
                        });
                    },
                    gtk::ResponseType::Cancel => dialog.hide(),
                    _ => ()
                }
            }),
        );

        let settings_btn_open_dir = builder.get::<gtk::Button>("settings_btn_open_dir");

        settings_btn_open_dir.connect_clicked(glib::clone!(@strong app_runtime => move |_| {
            app_runtime.update_state_with(move |state| {
                state.open_data_directory();
            });
        }));
    }

    pub fn update(&self, builder: &gtk::Builder, settings: &Settings) {
        debug!("Update settings dialog!");

        let mouse_2_action = self.action_comboboxes.get("mouse_2_action").unwrap();
        let mouse_3_action = self.action_comboboxes.get("mouse_3_action").unwrap();
        let mouse_4_action = self.action_comboboxes.get("mouse_4_action").unwrap();
        let mouse_5_action = self.action_comboboxes.get("mouse_5_action").unwrap();
        let reader = builder.get::<gtk::FileChooserButton>("reader_file");
        let reader_args = builder.get::<gtk::Entry>("reader_args");
        let settings_startup_auto = builder.get::<gtk::CheckButton>("settings_startup_auto");
        let settings_startup_minimized =
            builder.get::<gtk::CheckButton>("settings_startup_minimized");
        let settings_check_update =
            builder.get::<gtk::CheckButton>("settings_startup_check_update");
        let window_state_enabled = builder.get::<gtk::CheckButton>("settings_window_state_enabled");

        mouse_2_action.set_active_id(Some(&settings.general.mouse_2_action.to_i32().to_string()));
        mouse_3_action.set_active_id(Some(&settings.general.mouse_3_action.to_i32().to_string()));
        mouse_4_action.set_active_id(Some(&settings.general.mouse_4_action.to_i32().to_string()));
        mouse_5_action.set_active_id(Some(&settings.general.mouse_5_action.to_i32().to_string()));

        self.preference_combobox.set_active_id(Some(
            &settings
                .novel_recognition
                .chapter_read_preference
                .to_i32()
                .to_string(),
        ));

        let viscol_status = builder.get::<gtk::CheckButton>("viscol_status");
        let viscol_cco = builder.get::<gtk::CheckButton>("viscol_cco");
        let viscol_name = builder.get::<gtk::CheckButton>("viscol_name");
        let viscol_ch = builder.get::<gtk::CheckButton>("viscol_ch");
        let viscol_vol = builder.get::<gtk::CheckButton>("viscol_vol");
        let viscol_side = builder.get::<gtk::CheckButton>("viscol_side");
        let viscol_avail = builder.get::<gtk::CheckButton>("viscol_avail");
        let viscol_score = builder.get::<gtk::CheckButton>("viscol_score");
        let viscol_last = builder.get::<gtk::CheckButton>("viscol_last");
        let tab_always = builder.get::<gtk::CheckButton>("first_tab_always_checkbox");

        for (index, col) in settings.list.visible_columns.iter().enumerate() {
            match Column::from_i32(index as i32) {
                Column::Status => viscol_status.set_active(*col),
                Column::OriginalLanguage => viscol_cco.set_active(*col),
                Column::Title => viscol_name.set_active(*col),
                Column::ChaptersRead => viscol_ch.set_active(*col),
                Column::VolumesRead => viscol_vol.set_active(*col),
                Column::SideStoriesRead => viscol_side.set_active(*col),
                Column::ChaptersAvailable => viscol_avail.set_active(*col),
                Column::Score => viscol_score.set_active(*col),
                Column::LastUpdate => viscol_last.set_active(*col),
                _ => {}
            }
        }

        self.novel_info_tabs_combobox
            .set_active(Some(settings.list.open_info_behavior as u32));
        tab_always.set_active(settings.list.always_open_selected_tab);

        settings_startup_auto.set_active(settings.general.open_with_windows);
        settings_startup_minimized.set_active(settings.general.start_minimized);
        settings_check_update.set_active(settings.general.check_update);
        window_state_enabled.set_active(settings.general.window_state_enabled);

        if let Some(filename) = &settings.general.reader {
            reader.set_filename(filename);
        } else {
            reader.unselect_all();
        }
        reader_args.set_text(&settings.general.reader_args);

        if let Some(language) = &settings.general.language {
            self.language_combobox.set_active_id(Some(language));
        } else {
            self.language_combobox.set_active_id(Some("none"));
        }

        let novel_recognition_enabled_checkbutton =
            builder.get::<gtk::CheckButton>("novel_recognition_enabled_checkbutton");
        let novel_recognition_delay = builder.get::<gtk::SpinButton>("novel_recognition_delay");
        let novel_recognition_read_preference_combobox =
            builder.get::<gtk::ComboBoxText>("novel_recognition_read_preference_combobox");
        let novel_recognition_autocomplete_ongoing =
            builder.get::<gtk::CheckButton>("novel_recognition_autocomplete_ongoing");
        let novel_recognition_title_keywords_entry =
            builder.get::<gtk::Entry>("novel_recognition_title_keywords_entry");
        let novel_recognition_found_go_to_reading =
            builder.get::<gtk::CheckButton>("novel_rec_found_go_to_reading");
        let novel_recognition_not_found_go_to_reading =
            builder.get::<gtk::CheckButton>("novel_rec_not_found_go_to_reading");
        let novel_recognition_ignore_keywords_entry =
            builder.get::<gtk::Entry>("novel_recognition_ignore_keywords_entry");

        novel_recognition_enabled_checkbutton.set_active(settings.novel_recognition.enable);
        novel_recognition_delay.set_value(settings.novel_recognition.delay as f64);
        novel_recognition_read_preference_combobox.set_active_id(Some(
            settings
                .novel_recognition
                .chapter_read_preference
                .to_string()
                .as_str(),
        ));
        novel_recognition_autocomplete_ongoing
            .set_active(settings.novel_recognition.autocomplete_ongoing);
        novel_recognition_found_go_to_reading
            .set_active(settings.novel_recognition.when_novel_go_to_reading);
        novel_recognition_not_found_go_to_reading
            .set_active(settings.novel_recognition.when_not_novel_go_to_reading);
        novel_recognition_title_keywords_entry
            .set_text(&settings.novel_recognition.title_keywords.join(","));
        novel_recognition_ignore_keywords_entry
            .set_text(&settings.novel_recognition.ignore_keywords.join(","));

        let data_dir_label = builder.get::<gtk::Label>("data_dir_label");

        data_dir_label.set_label(settings.general.data_dir.to_str().unwrap());
    }
}

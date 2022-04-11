use crate::app::error::ErisError;
use crate::app::settings::{ChapterReadPreference, NovelListAction, Settings};
use crate::appop::AppOp;
use crate::utils::gtk::BuilderExtManualCustom;
use anyhow::Context;
use gtk::prelude::{
    CheckMenuItemExt, ComboBoxExt, EntryExt, FileChooserExt, SpinButtonExt, ToggleButtonExt, WidgetExt,
};
use parking_lot::RwLock;
use std::process::Command;
use std::sync::Arc;

impl AppOp {
    /// Change settings to their default values.
    pub fn set_default_settings(&mut self) {
        let settings = Settings::default();

        // Update ui and other settings relevant things
        self.ui.settings_dialog.update(&self.ui.builder, &settings);
        self.update_settings();

        // Write new settings to file
        settings
            .write_to_file()
            .context(ErisError::WriteToDisk)
            .expect("Failed to default the settings");

        // Update new settings in memory
        self.settings = Arc::new(RwLock::new(settings));
    }

    /// Hide or show the sidebar
    pub fn toggle_sidebar(&mut self) {
        let builder = &self.ui.builder;

        let view_selection_listbox = builder.get::<gtk::ListBox>("view_selection_listbox");

        view_selection_listbox.set_visible(!view_selection_listbox.is_visible());

        let mut settings = self.settings.write();
        settings.general.show_sidebar = view_selection_listbox.is_visible();
        settings.write_to_file().expect("Cannot write settings to file.");
    }

    pub fn update_settings(&mut self) {
        debug!("appop::update_settings");
        let builder = &self.ui.builder;

        let mouse_2_action = self.ui.settings_dialog.action_comboboxes.get("mouse_2_action").unwrap();
        let mouse_3_action = self.ui.settings_dialog.action_comboboxes.get("mouse_3_action").unwrap();
        let mouse_4_action = self.ui.settings_dialog.action_comboboxes.get("mouse_4_action").unwrap();
        let mouse_5_action = self.ui.settings_dialog.action_comboboxes.get("mouse_5_action").unwrap();
        let settings_startup_auto = builder.get::<gtk::CheckButton>("settings_startup_auto");
        let settings_startup_minimized = builder.get::<gtk::CheckButton>("settings_startup_minimized");
        let settings_startup_check_update = builder.get::<gtk::CheckButton>("settings_startup_check_update");

        let viscol_status = builder.get::<gtk::CheckButton>("viscol_status");
        let viscol_cco = builder.get::<gtk::CheckButton>("viscol_cco");
        let viscol_name = builder.get::<gtk::CheckButton>("viscol_name");
        let viscol_ch = builder.get::<gtk::CheckButton>("viscol_ch");
        let viscol_vol = builder.get::<gtk::CheckButton>("viscol_vol");
        let viscol_side = builder.get::<gtk::CheckButton>("viscol_side");
        let viscol_avail = builder.get::<gtk::CheckButton>("viscol_avail");
        let viscol_score = builder.get::<gtk::CheckButton>("viscol_score");
        let viscol_last = builder.get::<gtk::CheckButton>("viscol_last");
        let tab_behavior = &self.ui.settings_dialog.novel_info_tabs_combobox;

        let reader = builder.get::<gtk::FileChooserButton>("reader_file");
        let reader_args = builder.get::<gtk::Entry>("reader_args");
        let language = &self.ui.settings_dialog.language_combobox;

        let novel_recognition_enabled_checkbutton =
            builder.get::<gtk::CheckButton>("novel_recognition_enabled_checkbutton");
        let novel_recognition_delay = builder.get::<gtk::SpinButton>("novel_recognition_delay");
        let novel_recognition_read_preference_combobox =
            builder.get::<gtk::ComboBoxText>("novel_recognition_read_preference_combobox");
        let novel_recognition_autocomplete_ongoing =
            builder.get::<gtk::CheckButton>("novel_recognition_autocomplete_ongoing");
        let novel_recognition_title_keywords_entry =
            builder.get::<gtk::Entry>("novel_recognition_title_keywords_entry");
        let novel_recognition_ignore_keywords_entry =
            builder.get::<gtk::Entry>("novel_recognition_ignore_keywords_entry");
        let toggle_novel_recognition = builder.get::<gtk::CheckMenuItem>("toggle_novel_recognition");
        let novel_rec_found_go_to_reading = builder.get::<gtk::CheckButton>("novel_rec_found_go_to_reading");
        let novel_rec_not_found_go_to_reading = builder.get::<gtk::CheckButton>("novel_rec_not_found_go_to_reading");
        let window_state_enabled = builder.get::<gtk::CheckButton>("settings_window_state_enabled");

        toggle_novel_recognition.set_active(novel_recognition_enabled_checkbutton.is_active());

        let mut new_settings = self.settings.write().clone();
        new_settings.general.mouse_2_action =
            NovelListAction::from_i32(mouse_2_action.active_id().unwrap().parse::<i32>().unwrap_or(0));
        new_settings.general.mouse_3_action =
            NovelListAction::from_i32(mouse_3_action.active_id().unwrap().parse::<i32>().unwrap_or(0));
        new_settings.general.mouse_4_action =
            NovelListAction::from_i32(mouse_4_action.active_id().unwrap().parse::<i32>().unwrap_or(0));
        new_settings.general.mouse_5_action =
            NovelListAction::from_i32(mouse_5_action.active_id().unwrap().parse::<i32>().unwrap_or(0));

        new_settings.list.visible_columns = vec![
            false, // id
            viscol_status.is_active(),
            false, // status icon
            viscol_cco.is_active(),
            viscol_name.is_active(),
            viscol_ch.is_active(),
            viscol_side.is_active(),
            viscol_vol.is_active(),
            viscol_avail.is_active(),
            viscol_score.is_active(),
            viscol_last.is_active(),
        ];
        new_settings.list.open_info_behavior = tab_behavior.active_id().unwrap().parse().unwrap();
        new_settings.list.always_open_selected_tab =
            builder.get::<gtk::CheckButton>("first_tab_always_checkbox").is_active();

        new_settings.general.reader = reader.filename();
        new_settings.general.reader_args = reader_args.text().to_string();
        new_settings.general.open_with_windows = settings_startup_auto.is_active();
        new_settings.general.start_minimized = settings_startup_minimized.is_active();
        new_settings.general.check_update = settings_startup_check_update.is_active();
        new_settings.general.window_state_enabled = window_state_enabled.is_active();

        let selected_lang = language.active_id().unwrap().to_string();
        if selected_lang == "none" {
            new_settings.general.language = None;
        } else {
            new_settings.general.language = Some(selected_lang);
        }

        new_settings.novel_recognition.enable = novel_recognition_enabled_checkbutton.is_active();
        new_settings.novel_recognition.delay = novel_recognition_delay.value_as_int() as i64;
        new_settings.novel_recognition.chapter_read_preference = ChapterReadPreference::from_i32(
            novel_recognition_read_preference_combobox
                .active_id()
                .unwrap()
                .parse::<i32>()
                .unwrap(),
        );
        new_settings.novel_recognition.autocomplete_ongoing = novel_recognition_autocomplete_ongoing.is_active();
        new_settings.novel_recognition.when_novel_go_to_reading = novel_rec_found_go_to_reading.is_active();
        new_settings.novel_recognition.when_not_novel_go_to_reading = novel_rec_not_found_go_to_reading.is_active();

        let keywords: String = novel_recognition_title_keywords_entry.text().into();
        let keyword_vec: Vec<String> = keywords.split(',').map(|t| t.to_string()).collect();

        let ignore_keywords: String = novel_recognition_ignore_keywords_entry.text().into();
        let ignore_keyword_vec: Vec<String> = ignore_keywords.split(',').map(|t| t.trim().to_string()).collect();

        new_settings.novel_recognition.title_keywords = keyword_vec;
        new_settings.novel_recognition.ignore_keywords = ignore_keyword_vec;

        let old_settings = self.settings.read().clone();

        // If nothing was changed then do nothing
        if old_settings == new_settings {
            return;
        }

        self.app_runtime.update_state_with(move |state| {
            if old_settings.general.mouse_2_action != new_settings.general.mouse_2_action
                || old_settings.general.mouse_3_action != new_settings.general.mouse_3_action
                || old_settings.general.mouse_4_action != new_settings.general.mouse_4_action
                || old_settings.general.mouse_5_action != new_settings.general.mouse_5_action
            {
                // Update the novel list connect logic if the relevant settings changed.
                state
                    .ui
                    .lists
                    .connect_mouse_actions(state.app_runtime.clone(), &new_settings);
                state
                    .ui
                    .filter
                    .connect_mouse_actions(state.app_runtime.clone(), &new_settings);
                state
                    .ui
                    .history
                    .connect_mouse_actions(&state.ui.builder, state.app_runtime.clone(), &new_settings);
            }

            if old_settings.list.visible_columns != new_settings.list.visible_columns {
                // Update the columns visibility if any of them changed
                state.ui.lists.add_columns(state.app_runtime.clone(), &new_settings);
                state.ui.filter.add_columns(state.app_runtime.clone(), &new_settings);
            }

            if (old_settings.novel_recognition.title_keywords != new_settings.novel_recognition.title_keywords)
                || (old_settings.novel_recognition.ignore_keywords != new_settings.novel_recognition.ignore_keywords)
            {
                // Restart novel recognition thread if either keywords change, if it is running
                if state.novel_recognition.is_some() {
                    state.restart_novel_recognition(new_settings.clone());
                }
            }

            if old_settings.general.open_with_windows != new_settings.general.open_with_windows {
                #[cfg(target_os = "windows")]
                state.start_with_windows(new_settings.general.open_with_windows);
            }

            // Write new settings to file
            new_settings
                .write_to_file()
                .expect("Cannot write to the settings file.");
            // In memory, replace the old settings with the new ones
            state.settings = Arc::new(RwLock::new(new_settings));
        });
    }

    #[cfg(target_os = "windows")]
    pub fn open_data_directory(&mut self) {
        debug!("Opening dir: {:?}", self.settings.read().general.data_dir);
        Command::new("explorer")
            .arg(&self.settings.read().general.data_dir)
            .spawn()
            .unwrap();
    }

    #[cfg(target_os = "linux")]
    pub fn open_data_directory(&mut self) {
        Command::new("xdg-open")
            .arg(&self.settings.read().general.data_dir)
            .spawn()
            .unwrap();
    }

    #[cfg(target_os = "macos")]
    pub fn open_data_directory(&mut self) {
        Command::new("open")
            .arg(&self.settings.read().general.data_dir)
            .spawn()
            .unwrap();
    }

    #[cfg(target_os = "windows")]
    pub fn start_with_windows(&self, add_to_reg: bool) {
        use crate::APPLICATION_ID;
        use std::env::current_exe;
        use std::path::Path;
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = Path::new("Software")
            .join("Microsoft")
            .join("Windows")
            .join("CurrentVersion")
            .join("Run");
        let (key, _disp) = hkcu.create_subkey(&path).unwrap();

        if add_to_reg {
            key.set_value(
                &APPLICATION_ID.to_string(),
                &current_exe().unwrap().to_str().unwrap().to_string(),
            )
            .unwrap();
        } else {
            key.delete_value(APPLICATION_ID).unwrap();
        }
    }
}

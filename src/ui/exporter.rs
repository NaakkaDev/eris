use crate::app::AppRuntime;
use crate::ui::UI;
use gtk::prelude::{DialogExt, GtkWindowExt, WidgetExt};
use gtk::{ButtonsType, DialogFlags, MessageType, ResponseType};

impl UI {
    pub fn open_post_export_message(&self, app_runtime: &AppRuntime) {
        cascade! {
            gtk::MessageDialog::new(
                Some(&self.main_window),
                DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Question,
                ButtonsType::Ok,
                &fl!("exported-text")
            );
            ..add_button(&fl!("open-file-location"), ResponseType::Accept);
            ..set_title(&fl!("success"));
            ..connect_response(glib::clone!(@strong app_runtime => move |dialog, response_type| {
                match response_type {
                    gtk::ResponseType::Ok => {
                        dialog.close();
                    },
                    gtk::ResponseType::Accept => {
                        dialog.close();
                        // Open data dir
                        app_runtime.update_state_with(|state| {
                            state.open_data_directory();
                        });
                    }
                    _ => {}
                }
            }));
            ..show_all();
        };
    }

    pub fn open_post_import_message(&self, app_runtime: &AppRuntime) {
        cascade! {
            gtk::MessageDialog::new(
                Some(&self.main_window),
                DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Question,
                ButtonsType::None,
                &fl!("import-text")
            );
            ..add_button(&fl!("quit"), ResponseType::Accept);
            ..set_title(&fl!("success"));
            ..connect_response(glib::clone!(@strong app_runtime => move |dialog, response_type| {
                if response_type == gtk::ResponseType::Accept {
                    dialog.close();
                    // Quit app
                    app_runtime.update_state_with(|state| {
                        state.ui.main_window.close();
                    });
                }
            }));
            ..show_all();
        };
    }
}

use super::UI;
use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageType, ResponseType};

impl UI {
    /// Notification dialog for when manure hits the airing unit.
    pub fn notification_dialog(&self, message: &str) {
        cascade! {
            gtk::MessageDialog::new(
                Some(&self.main_window),
                DialogFlags::DESTROY_WITH_PARENT,
                MessageType::Warning,
                ButtonsType::Ok,
                message
            );
            ..set_title("Nom om-nom nom nom.. oops!");
            ..show();
            ..connect_delete_event(|dialog, _| {
                dialog.hide();
                gtk::Inhibit(true)
            });
            ..connect_response(|dialog, response| {
                if response == ResponseType::Ok {
                    dialog.close();
                }
            });
        };
    }
}

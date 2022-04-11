use super::UI;
use gdk::gdk_pixbuf::Pixbuf;
use std::io::Cursor;

use crate::utils::Resources;
use crate::VERSION;
use gtk::prelude::*;

impl UI {
    pub fn about_dialog(&self) {
        let resource = Resources::get("icons/eris_logo.png").unwrap().data;
        let icon_pix = Pixbuf::from_read(Cursor::new(resource)).expect("Cannot load pixbuf from resource.");

        let dialog = cascade! {
            gtk::AboutDialog::new();
            ..set_comments(Some(&fl!("about-text")));
            ..set_modal(true);
            ..set_version(Some(VERSION));
            ..set_program_name("Eris");
            ..set_logo(Some(&icon_pix));
            ..set_transient_for(Some(&self.main_window));
            ..connect_response(move |d, _| {
                d.close();
            });
        };

        dialog.show();
    }
}

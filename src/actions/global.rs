use gio::prelude::*;
use gio::SimpleAction;

use crate::appop::AppOp;
use gtk::prelude::{GtkApplicationExt, GtkWindowExt, WidgetExt};

pub fn new(appop: &AppOp) {
    let app = &appop.ui.gtk_app;
    let app_runtime = appop.app_runtime.clone();

    let about = SimpleAction::new("about", None);
    let quit = SimpleAction::new("quit", None);
    let new = SimpleAction::new("new", None);
    let save = SimpleAction::new("save", None);
    let settings = SimpleAction::new("settings", None);
    let toggle_maximize = SimpleAction::new("toggle_maximize", None);
    let show_sidebar = SimpleAction::new("show_sidebar", None);
    let show_reading_now = SimpleAction::new("show_reading_now", None);
    let show_novel_list = SimpleAction::new("show_novel_list", None);
    let show_history = SimpleAction::new("show_history", None);
    let toggle_novel_recognition = SimpleAction::new("toggle_novel_recognition", None);
    let select_search_entry = SimpleAction::new("select_search_entry", None);
    let export_db = SimpleAction::new("export_db", None);
    let export_history = SimpleAction::new("export_history", None);
    let update_menu = SimpleAction::new("update_menu", None);

    app.add_action(&about);
    app.add_action(&quit);
    app.add_action(&new);
    app.add_action(&save);
    app.add_action(&show_reading_now);
    app.add_action(&settings);
    app.add_action(&toggle_maximize);
    app.add_action(&show_sidebar);
    app.add_action(&show_novel_list);
    app.add_action(&show_history);
    app.add_action(&toggle_novel_recognition);
    app.add_action(&select_search_entry);
    app.add_action(&export_db);
    app.add_action(&export_history);
    app.add_action(&update_menu);

    app.set_accels_for_action("app.toggle_maximize", &["F11"]);
    app.set_accels_for_action("app.select_search_entry", &["<Primary>F"]);

    about.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| state.ui.about_dialog());
    }));

    quit.connect_activate(glib::clone!(@strong app_runtime => move |_action, _param| {
        app_runtime.update_state_with(|state| {
            state.ui.main_window.close();
        });
    }));

    new.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.show_new_dialog();
        });
    }));

    save.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.save_to_file();
        })
    }));

    settings.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.show_settings_dialog(state.settings.read().clone());
        });
    }));

    toggle_maximize.connect_activate(glib::clone!(@weak app => move |_, _| {
        if let Some(window) = app.active_window() {
            if window.is_maximized() {
                window.unmaximize();
            } else {
                window.maximize();
            }
        }
    }));

    show_sidebar.connect_activate(glib::clone!(@strong app_runtime =>  move |_, _| {
        app_runtime.update_state_with(|state| {
            state.toggle_sidebar();
        });
    }));

    show_reading_now.connect_activate(glib::clone!(@strong app_runtime =>  move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.show_reading_now();
        });
    }));

    show_novel_list.connect_activate(glib::clone!(@strong app_runtime =>  move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.show_novel_list()
        });
    }));

    show_history.connect_activate(glib::clone!(@strong app_runtime =>  move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.show_history();
        });
    }));

    toggle_novel_recognition.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.toggle_novel_recognition();
        });
    }));

    select_search_entry.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.filter.entry.grab_focus();
        });
    }));

    export_db.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            match state.export_db_to_json(&state.db.read()) {
                Ok(_) => {
                    state.ui.open_post_export_message(&state.app_runtime);
                },
                Err(e) => {
                    error!("Could not export db to json. {:?}", e);
                }
            }
        });
    }));

    export_history.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            match state.export_history_to_json(&state.history.read()) {
                Ok(_) => {
                    state.ui.open_post_export_message(&state.app_runtime);
                },
                Err(e) => {
                    error!("Could not export history to json. {:?}", e);
                }
            }
        });
    }));

    update_menu.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.open_update_link();
        });
    }));
}

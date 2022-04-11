use gio::prelude::*;
use glib::clone;
use gtk::gdk;
use gtk::prelude::*;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::actions;
use crate::app::window_state::WindowState;
use crate::appop::AppOp;
use crate::ui;
use crate::utils::Resources;

pub mod database;
pub mod error;
pub mod history;
pub mod localize;
pub mod novel;
pub mod settings;
pub mod window_state;

pub const NOVEL_UPDATE_COOLDOWN: i64 = 3600;

#[derive(Clone)]
pub struct AppRuntime(glib::Sender<Box<dyn FnOnce(&mut AppOp)>>);

impl AppRuntime {
    fn init(ui: ui::UI) -> Self {
        let (app_tx, app_rx) = glib::MainContext::channel(Default::default());
        let app_runtime = Self(app_tx);
        let mut state = AppOp::new(ui, app_runtime.clone());

        app_rx.attach(None, move |update_state| {
            update_state(&mut state);

            glib::Continue(true)
        });

        debug!("app::AppRuntime::init");

        app_runtime
    }

    pub fn update_state_with(&self, update_fn: impl FnOnce(&mut AppOp) + 'static) {
        let _ = self.0.send(Box::new(update_fn));
    }
}

fn new(gtk_app: gtk::Application) -> AppRuntime {
    glib::set_application_name("eris");
    glib::set_prgname(Some("eris"));

    let css_file = Resources::get("css/app.css").unwrap();
    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(&css_file.data)
        .expect("Cannot load app.css file");

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let ui = ui::UI::new(gtk_app);
    let app_runtime = AppRuntime::init(ui);

    app_runtime.update_state_with(move |state| {
        if state.settings.read().general.window_state_enabled {
            let window_state = WindowState::open();
            if let Some(window_state) = window_state.unwrap() {
                // Set the window size which are loaded from the state file
                state
                    .ui
                    .main_window
                    .set_default_size(window_state.width, window_state.height);
                // Set the window position
                if window_state.x != 0 || window_state.y != 0 {
                    state.ui.main_window.move_(window_state.x, window_state.y);
                }
                // Make the window maximized if it was previously
                if window_state.is_maximized {
                    state.ui.main_window.maximize();
                }

                // Save the loaded window state to `AppOp`
                state.window_state = Arc::new(RwLock::new(Some(window_state)));
            }
        }

        state.ui.init();
        state
            .ui
            .settings_dialog
            .update(&state.ui.builder, &state.settings.read());
        state.ui.init_menu(&state.settings.read());

        state.ui.connect(state.app_runtime.clone());
        state
            .ui
            .lists
            .connect(&state.ui.builder, state.app_runtime.clone(), &state.settings.read());
        state
            .ui
            .filter
            .connect(state.app_runtime.clone(), &state.ui.list_notebook);
        state.ui.history.connect(&state.ui.builder, state.app_runtime.clone());
        state
            .ui
            .lists
            .connect_mouse_actions(state.app_runtime.clone(), &state.settings.read());
        state
            .ui
            .filter
            .connect_mouse_actions(state.app_runtime.clone(), &state.settings.read());
        state
            .ui
            .history
            .connect_mouse_actions(&state.ui.builder, state.app_runtime.clone(), &state.settings.read());
        state
            .ui
            .novel_dialog
            .connect(&state.ui.builder, state.app_runtime.clone());
        state
            .ui
            .new_dialog
            .connect(&state.ui.builder, state.app_runtime.clone(), state.ui.url_list.clone());
        state
            .ui
            .file_new_dialog
            .connect(&state.ui.builder, state.app_runtime.clone(), state.ui.url_list.clone());
        state
            .ui
            .settings_dialog
            .connect(&state.ui.builder, state.app_runtime.clone());

        actions::Global::new(state);
    });

    debug!("app::new");

    app_runtime
}

pub fn on_startup(gtk_app: &gtk::Application) {
    let app_runtime = new(gtk_app.clone());

    debug!("app::on_startup");

    gtk_app.connect_activate(clone!(@strong app_runtime => move |_| {
        app_runtime.update_state_with(|state| {
            if state.settings.read().general.start_minimized {
                state.ui.main_window.iconify();
                state.ui.main_window.show();
            } else {
                on_activate(&state.ui);
            }
            // Check for update on startup if enabled
            if state.settings.read().general.check_update {
                state.check_for_update(&state.app_runtime);
            }
        });
    }));

    app_runtime.update_state_with(|state| {
        // Do things when the main window gets the delete event
        // so when it's being closed.
        // Does not work with ALT+F4 though.
        let old_window_state = state.window_state.read().clone();
        let window_state_enabled = state.settings.read().general.window_state_enabled;
        state.ui.main_window.connect_delete_event(move |window, _| {
            if window_state_enabled {
                let window = window.upcast_ref();
                // Get the current window state and save it to file
                let window_state = WindowState::from_window_with_old_values(window, old_window_state.clone());
                if let Err(err) = window_state.write_to_file() {
                    error!("Cannot save the window state to file! {:?}", err);
                }
            }

            Inhibit(false)
        });

        state.init();
    });

    gtk_app.connect_shutdown(move |_| {
        app_runtime.update_state_with(|state| {
            on_shutdown(state);
        });
    });
}

fn on_activate(ui: &ui::UI) {
    ui.main_window.show();
    ui.main_window.present();

    debug!("app::on_activate");
}

fn on_shutdown(appop: &AppOp) {
    debug!("app::on_shutdown");

    // Save settings to file on quit
    appop
        .settings
        .write()
        .write_to_file()
        .expect("Cannot save settings to file");
    // Quit
    appop.quit();
}

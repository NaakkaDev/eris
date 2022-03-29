#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate anyhow;
extern crate epub;
extern crate select;
extern crate thiserror;
extern crate ureq;
extern crate window_titles;
#[macro_use]
extern crate cascade;
extern crate bincode;
extern crate chrono;
extern crate sanitize_filename;
extern crate serde;
extern crate slug;
extern crate webbrowser;
#[macro_use]
extern crate log;

use const_format::formatcp;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use crate::app::localize::localizer;
use crate::app::settings::Settings;
use crate::utils::data_dir;
use gtk::prelude::*;
use i18n_embed::unic_langid::LanguageIdentifier;
use i18n_embed::DesktopLanguageRequester;

#[macro_use]
mod utils;
pub mod actions;
pub mod app;
pub mod appop;
pub mod ui;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const UPDATE_LINK: &str = formatcp!("{}releases/latest", env!("CARGO_PKG_REPOSITORY"));

pub const DATA_DIR: &str = "data";
pub const LOG_FILE: &str = formatcp!("{DATA_DIR}/eris.log");
pub const CONFIG_NAME: &str = formatcp!("{DATA_DIR}/eris.conf");
pub const DB_FILE: &str = formatcp!("{DATA_DIR}/db/eris.db");
pub const HISTORY_FILE: &str = formatcp!("{DATA_DIR}/eris.history");
pub const STATE_CONFIG_NAME: &str = formatcp!("{DATA_DIR}/eris.state");
pub const DATA_IMAGE_DIR: &str = formatcp!("{DATA_DIR}/db/images");
pub const APPLICATION_ID: &str = "com.github.temeez.eris";

fn setup_logging() -> Result<(), fern::InitError> {
    #[cfg(debug_assertions)]
    let log_level = log::LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    let log_level = log::LevelFilter::Info;

    let path = Path::new(&data_dir(LOG_FILE)).to_path_buf();

    // Clear the log file if it exists
    if path.as_path().exists() {
        let f = File::create(&path).expect("Cannot open log file");
        f.set_len(0).expect("Cannot set file length to 0");
    }

    let settings_data = Settings::open().expect("Failed to open settings file.");
    // Get either the language from settings or the system default one
    let requested_languages: Vec<LanguageIdentifier> =
        if let Some(language) = &settings_data.general.language {
            vec![LanguageIdentifier::from_str(language).unwrap()]
        } else {
            DesktopLanguageRequester::requested_languages()
        };

    if let Err(error) = localizer().select(&requested_languages) {
        error!("Cannot load language. {:?}", error);
    }

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("eris", log_level)
        .chain(std::io::stdout())
        .chain(fern::log_file(path)?)
        .apply()?;

    Ok(())
}

fn generate_dirs() {
    if !&data_dir(DATA_IMAGE_DIR).exists() {
        match fs::create_dir_all(&data_dir(DATA_IMAGE_DIR)) {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e);
                panic!(
                    "Could not create directory `{}`",
                    &data_dir(DATA_IMAGE_DIR).to_str().unwrap()
                );
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Verify that the folder exists and create it if not
    generate_dirs();
    // The settings file location needs to exist for `setup_logging()`
    setup_logging().expect("failed to initialize logging.");

    let application =
        gtk::Application::new(Some(APPLICATION_ID), gio::ApplicationFlags::FLAGS_NONE);

    application.connect_startup(|application| {
        app::on_startup(application);
    });

    application.run();

    Ok(())
}

pub mod general;
pub mod list;
pub mod novel_recognition;

pub use general::{GeneralSettings, NovelListAction};
pub use list::{ListSettings, Sorting};
pub use novel_recognition::{ChapterReadPreference, NovelRecognitionSettings};

use crate::app::error::ErisError;
use crate::{data_dir, CONFIG_NAME};
use anyhow::Context;
use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename = "settings")]
pub struct Settings {
    /// Settings specific to novel lists.
    pub list: ListSettings,
    /// Application settings.
    pub general: GeneralSettings,
    /// Novel recognition settings.
    pub novel_recognition: NovelRecognitionSettings,
    /// Filepath to the settings file.
    path: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        let path = data_dir(CONFIG_NAME);

        Settings {
            list: ListSettings::default(),
            general: GeneralSettings::default(),
            novel_recognition: NovelRecognitionSettings::default(),
            path,
        }
    }
}

impl Settings {
    pub fn open() -> Result<Self, ErisError> {
        let path = &data_dir(CONFIG_NAME);
        if path.exists() {
            let f = File::open(path).context(ErisError::ReadFromDisk)?;
            let reader = BufReader::new(f);

            if let Ok(settings) = deserialize_from(reader).context(ErisError::Unknown) {
                return Ok(settings);
            }
        }

        // If file doesn't exist then set then create it
        let settings = Settings::default();

        settings.write_to_file().context(ErisError::WriteToDisk)?;
        Ok(settings)
    }

    pub fn write_to_file(&self) -> Result<(), ErisError> {
        debug!("Saving settings to file.");

        let f = File::create(&self.path).unwrap();
        let writer = BufWriter::new(f);

        serialize_into(writer, self).context(ErisError::WriteToDisk)?;

        Ok(())
    }
}

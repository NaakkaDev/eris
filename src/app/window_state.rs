use gtk::prelude::*;

use crate::app::error::ErisError;
use crate::{data_dir, STATE_CONFIG_NAME};
use anyhow::Context;
use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_maximized: bool,
}

impl Default for WindowState {
    fn default() -> WindowState {
        WindowState {
            x: 300,
            y: 300,
            width: 730,
            height: 600,
            is_maximized: false,
        }
    }
}

impl WindowState {
    pub fn from_window(window: &gtk::ApplicationWindow) -> WindowState {
        let is_maximized = window.is_maximized();
        let position = window.position();
        let size = window.size();
        let x = position.0;
        let y = position.1;
        let width = size.0;
        let height = size.1;

        WindowState {
            x,
            y,
            width,
            height,
            is_maximized,
        }
    }

    /// Returns `WindowState` with old size and position values
    /// if the current window is maximized.
    pub fn from_window_with_old_values(window: &gtk::ApplicationWindow, old_state: Option<WindowState>) -> WindowState {
        if let Some(old_values) = old_state {
            let is_maximized = window.is_maximized();
            if is_maximized {
                return WindowState {
                    is_maximized,
                    ..old_values
                };
            }

            // Check if the window position is minus hell (minified)
            // if so then do not save those value or the window will
            // try to open at -32xxx coords which is not within the monitor viewports
            let position = window.position();
            let x = position.0;
            let y = position.1;

            if x < 0 && y < 0 {
                return WindowState { ..old_values };
            }
        }

        WindowState::from_window(window)
    }

    pub fn write_to_file(&self) -> Result<(), ErisError> {
        let path = data_dir(STATE_CONFIG_NAME);
        let f = File::create(&path).unwrap();
        let writer = BufWriter::new(f);

        serialize_into(writer, self).context(ErisError::WriteToDisk)?;

        Ok(())
    }

    pub fn open() -> Result<Option<Self>, ErisError> {
        let path = data_dir(STATE_CONFIG_NAME);
        if path.as_path().exists() {
            let f = File::open(&path).context(ErisError::ReadFromDisk)?;
            let reader = BufReader::new(f);
            if let Ok(settings) = deserialize_from(reader).context(ErisError::Unknown) {
                return Ok(Some(settings));
            }
        }

        Ok(Some(Self::default()))
    }
}

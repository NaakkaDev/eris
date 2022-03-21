pub(crate) mod gtk;

use rust_embed::RustEmbed;
use std::env::current_exe;
use std::path::PathBuf;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

macro_rules! vec_string {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::app::localize::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::app::localize::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

#[derive(RustEmbed)]
#[folder = "resources/"]
pub struct Resources;

pub fn working_dir(file: &str) -> String {
    #[cfg(debug_assertions)]
    let pop_count = 3;

    #[cfg(all(not(debug_assertions), target_os = "windows"))]
    let pop_count = 2;

    #[cfg(all(not(debug_assertions), target_os = "linux"))]
    let pop_count = 1;

    #[cfg(all(not(debug_assertions), target_os = "macos"))]
    let pop_count = 1;

    let dir = match current_exe() {
        Ok(mut path) => {
            for _ in 0..pop_count {
                path.pop();
            }
            path
        }
        Err(e) => {
            // Is this even possible?
            error!("Could not get current_exe path. {:?}", e);
            PathBuf::new()
        }
    };

    if let Some(filepath) = dir.join(file).to_str() {
        return filepath.to_string();
    }

    "".to_string()
}

#[cfg(debug_assertions)]
pub fn data_dir(file: &str) -> PathBuf {
    PathBuf::from(working_dir(file))
}

#[cfg(not(debug_assertions))]
pub fn data_dir(file: &str) -> PathBuf {
    // Use local data dir if it exists
    if PathBuf::from(&working_dir("data")).exists() {
        return PathBuf::from(&working_dir(file));
    }
    // Otherwise use OS specific "data" directory
    if let Some(dir) = dirs::data_dir() {
        return dir.join("Eris").join(file);
    }

    return PathBuf::new();
}

/// Check if the string is empty (or "0") and return "-" if true
/// otherwise return the actual string.
/// This is for GUI so the value field has something instead of emptiness.
pub fn nil_str(value: &str) -> String {
    if value.is_empty() || value == "0" {
        return "-".to_string();
    }

    value.to_string()
}

pub fn capitalize_str(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

use crate::app::database::Database;
use crate::app::error::ErisError;
use crate::app::history::NovelHistory;
use crate::appop::AppOp;
use crate::data_dir;
use anyhow::Context;
use chrono::Local;
use parking_lot::lock_api::RwLock;
use serde_json::Result;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

/// Change this if `Database` structure changes.
const DB_VERSION: &str = "1.0";
/// Change this if `NovelHistory` structure changes.
const HISTORY_VERSION: &str = "1.0";

impl AppOp {
    pub fn export_db_to_json(&self, db: &Database) -> Result<()> {
        let json_data = serde_json::to_string(db)?;
        let json_file_name = format!("data/db_v{}_{}.json", DB_VERSION, Local::now().timestamp());

        let path = data_dir(&json_file_name);
        let file_result = File::create(&path).context(ErisError::WriteToDisk);
        match file_result {
            Ok(f) => {
                let mut writer = BufWriter::new(f);
                let _ = write!(&mut writer, "{}", json_data).context(ErisError::WriteToDisk);
                debug!("DB exported to json. Filename: {:?}", json_file_name);
            }
            Err(e) => {
                panic!("Cannot write DB to json file. {}", e);
            }
        }

        Ok(())
    }

    /// Import `Database` from a JSON file. Empty file throws an expect and
    /// a valid JSON file with invalid fields results in `Database::default()`.
    pub fn import_json_to_db(&mut self, filename: String) -> Result<()> {
        let json_file = Path::new(&filename);
        if json_file.exists() {
            let json_data = fs::read_to_string(&json_file).expect("Unable to read JSON file.");
            let mut db: Database = serde_json::from_str(&json_data).expect("Unable to parse JSON.");

            // Save to file
            match db.write_to_file() {
                Ok(_) => {}
                Err(e) => {
                    error!("Cannot write to db file. {:?}", e);
                }
            }
            // Save to memory
            self.db = Arc::new(RwLock::new(db));
        }

        Ok(())
    }

    // pub fn import_json_to_db_custom(&mut self, filename: String) {
    //     let json_file = Path::new(&filename);
    //     if json_file.exists() {
    //         let root: serde_json::Value = serde_json::from_str(&json_data).expect("Unable to parse JSON.");
    //         for novel in root.get("novel") {
    //             println!("novel: {:?}", novel);
    //         }
    //     }
    // }

    pub fn export_history_to_json(&self, history: &NovelHistory) -> Result<()> {
        let json_data = serde_json::to_string(history)?;
        let json_file_name = format!(
            "data/history_v{}_{}.json",
            HISTORY_VERSION,
            Local::now().timestamp()
        );

        let file_result = File::create(&data_dir(&json_file_name)).context(ErisError::WriteToDisk);
        match file_result {
            Ok(file) => {
                let mut f = BufWriter::new(file);
                let _ = write!(&mut f, "{}", json_data).context(ErisError::WriteToDisk);
                debug!("History exported to json. Filename: {:?}", json_file_name);
            }
            Err(e) => {
                panic!("Cannot write History to json file. {}", e);
            }
        }

        Ok(())
    }

    pub fn import_json_to_history(&mut self, filename: String) -> Result<()> {
        let json_file = Path::new(&filename);
        if json_file.exists() {
            let json_data = fs::read_to_string(&json_file).expect("Unable to read JSON file.");
            let history: NovelHistory =
                serde_json::from_str(&json_data).expect("Unable to parse JSON.");

            // Save to file
            match history.write_to_file() {
                Ok(_) => {}
                Err(e) => {
                    error!("Cannot write to history file. {:?}", e);
                }
            }
            // Save to memory
            self.history = Arc::new(RwLock::new(history));
        }

        Ok(())
    }

    /// Likely needs updating if `Database` or `NovelHistory` structs ever change.
    pub fn import_json(&mut self, file_path: String) -> Result<()> {
        let filename = Path::new(&file_path).file_name().unwrap().to_str().unwrap();

        match filename.split('_').into_iter().collect::<Vec<&str>>()[1] {
            "v1.0" => {
                if filename.contains("history") {
                    return self.import_json_to_history(file_path);
                } else if filename.contains("db") {
                    return self.import_json_to_db(file_path);
                } else {
                    warn!("JSON file name not recognized.");
                }
            }
            "v1.1" => {
                warn!("Version 1.1 not implemented.");
            }
            _ => {
                warn!("Not implemented.");
            }
        }

        Ok(())
    }
}

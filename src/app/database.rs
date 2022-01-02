use std::fs::File;
use std::io::{BufReader, BufWriter};

use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};

use crate::app::error::ErisError;
use crate::app::novel::Novel;
use crate::{data_dir, DB_FILE};
use anyhow::Context;
use chrono::Local;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "database", default)]
pub struct Database {
    #[serde(rename = "novel")]
    pub novels: Option<Vec<Novel>>,
    /// Last time the database file was saved.
    pub last_update: i64,
}
impl Default for Database {
    fn default() -> Self {
        Self::new(Some(vec![]))
    }
}

impl Database {
    pub fn new(novels: Option<Vec<Novel>>) -> Database {
        Database {
            novels,
            last_update: Local::now().timestamp(),
        }
    }

    /// Push a new `Novel` into the database vector of novels.
    pub fn push_novel(&mut self, novel: Novel) {
        if let Some(novels) = &mut self.novels {
            novels.push(novel);
        }
    }
    /// Serialize and write database into file.
    pub fn write_to_file(&mut self) -> Result<(), anyhow::Error> {
        self.last_update = Local::now().timestamp();

        let path = data_dir(DB_FILE);
        let f = File::create(&path).context(ErisError::WriteToDisk)?;
        let writer = BufWriter::new(f);

        serialize_into(writer, self).context(ErisError::SerializeToFile)
    }

    /// Try to deserialize file contents into `Database`
    fn from_file(f: &File) -> Self {
        let reader = BufReader::new(f);
        match deserialize_from(reader).context(ErisError::DeserializeFromFile) {
            Ok(db) => db,
            Err(e) => panic!("`{}`: {}", DB_FILE, e),
        }
    }
}

/// Try to deserialize the database from the file or create a new
/// databse instance if the file doesn't exist.
pub(crate) fn read_database() -> Database {
    let path = data_dir(DB_FILE);
    debug!("Database file exists: {:?}", &path.exists());

    // Create the db file if it doesn't exist
    if !&path.exists() {
        return Database::new(Some(vec![]));
    }

    // Open the db file and try to deserialize its contents into `Database`
    match File::open(&path).context(ErisError::ReadFromDisk) {
        Ok(file) => Database::from_file(&file),
        Err(e) => panic!("`{}`: {}", DB_FILE, e),
    }
}

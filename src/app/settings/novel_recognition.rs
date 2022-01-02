use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ChapterReadPreference {
    /// Set the previous chapter number as read.
    Previous,
    /// Set the current chapter number as read.
    Current,
}

impl ToString for ChapterReadPreference {
    fn to_string(&self) -> String {
        match *self {
            ChapterReadPreference::Previous => fl!("previous"),
            ChapterReadPreference::Current => fl!("current"),
        }
    }
}

impl FromStr for ChapterReadPreference {
    type Err = ();

    fn from_str(input: &str) -> Result<ChapterReadPreference, Self::Err> {
        match input {
            "Previous" => Ok(ChapterReadPreference::Previous),
            "Current" => Ok(ChapterReadPreference::Current),
            _ => Err(()),
        }
    }
}

impl ChapterReadPreference {
    pub fn vec() -> Vec<String> {
        vec![
            ChapterReadPreference::Previous.to_string(),
            ChapterReadPreference::Current.to_string(),
        ]
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => ChapterReadPreference::Previous,
            _ => ChapterReadPreference::Current,
        }
    }

    pub fn to_i32(&self) -> i32 {
        self.to_owned() as i32
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NovelRecognitionSettings {
    /// Enable novel recognition system.
    pub enable: bool,
    /// Delay in seconds. How long the recognized novel has to be
    /// "active" before the chapter is recorded.
    pub delay: i64,
    /// Should the current chapter be saved as read,
    /// or perhaps the previous one.
    pub chapter_read_preference: ChapterReadPreference,
    /// When novel is recognized go to Reading Now view or not.
    pub when_novel_go_to_reading: bool,
    /// When novel is not recognized go to Reading Now view or not.
    pub when_not_novel_go_to_reading: bool,
    /// Strings to look for from window titles.
    pub title_keywords: Vec<String>,
    /// Ignore window titles with these words.
    pub ignore_keywords: Vec<String>,
}

impl Default for NovelRecognitionSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl NovelRecognitionSettings {
    pub fn new() -> Self {
        let enable = true;
        let chapter_read_preference = ChapterReadPreference::Current;
        let title_keywords = vec_string!["Chapter", "Novel Updates", "Royal Road", "Scribble Hub"];
        let ignore_keywords = vec_string!["Manga", "Manhua", "Manhwa"];

        NovelRecognitionSettings {
            enable,
            delay: 120,
            chapter_read_preference,
            when_novel_go_to_reading: true,
            when_not_novel_go_to_reading: true,
            title_keywords,
            ignore_keywords,
        }
    }
}

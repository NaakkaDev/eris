use crate::app::error::ErisError;
use crate::app::novel::{Novel, NovelContentAmount};
use crate::ui::novel_list::ListStatus;
use crate::{data_dir, HISTORY_FILE};
use anyhow::Context;
use bincode::{deserialize_from, serialize_into};
use chrono::{Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum HistoryAction {
    NovelAdd,
    NovelDelete,
    NovelUpdate,
    NovelListChange,
    ContentRead,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NovelHistoryItem {
    pub novel_id: String,
    pub novel_name: String,
    pub action: HistoryAction,
    pub content: Option<NovelContentAmount>,
    pub list_status: Option<ListStatus>,
    pub named_chapter: Option<String>,
    pub time: i64,
}

impl NovelHistoryItem {
    pub fn new(
        novel_id: String,
        novel_name: String,
        action: HistoryAction,
        content: Option<NovelContentAmount>,
        list_status: Option<ListStatus>,
        named_chapter: Option<String>,
    ) -> Self {
        NovelHistoryItem {
            novel_id,
            novel_name,
            action,
            content,
            list_status,
            named_chapter,
            time: Local::now().timestamp_millis(),
        }
    }

    /// Internal function for creating a new `NovelHistoryItem` and returning it.
    fn add_item(
        novel_id: String,
        novel_name: String,
        action: HistoryAction,
        content: Option<NovelContentAmount>,
        list_status: Option<ListStatus>,
        named_chapter: Option<String>,
    ) -> Self {
        NovelHistoryItem::new(
            novel_id,
            novel_name,
            action,
            content,
            list_status,
            named_chapter,
        )
    }

    /// Adds a new history record for when adding a new novel.
    pub fn new_history_add_novel(novel: &Novel) -> Self {
        NovelHistoryItem::add_item(
            novel.id.clone(),
            novel.title.clone(),
            HistoryAction::NovelAdd,
            None,
            None,
            None,
        )
    }

    /// Adds a new history record for when deleting a novel.
    pub fn new_history_delete_novel(novel: &Novel) -> Self {
        NovelHistoryItem::add_item(
            novel.id.clone(),
            novel.title.clone(),
            HistoryAction::NovelDelete,
            None,
            None,
            None,
        )
    }

    /// Adds a new history record for when updaing a novel.
    pub fn new_history_update_novel(novel: &Novel) -> Self {
        NovelHistoryItem::add_item(
            novel.id.clone(),
            novel.title.clone(),
            HistoryAction::NovelUpdate,
            None,
            None,
            None,
        )
    }

    /// Adds a new history record for when novel list changes.
    pub fn new_history_novel_list_change(novel: &Novel) -> Self {
        NovelHistoryItem::add_item(
            novel.id.clone(),
            novel.title.clone(),
            HistoryAction::NovelListChange,
            None,
            Some(novel.settings.list_status),
            None,
        )
    }

    /// Adds a new history record for when the read volume or chapter changes in the novel settings.
    pub fn new_history_chapter_read(novel: &Novel, chapter_title: Option<String>) -> Self {
        NovelHistoryItem::add_item(
            novel.id.clone(),
            novel.title.clone(),
            HistoryAction::ContentRead,
            Some(novel.settings.content_read.clone()),
            None,
            chapter_title,
        )
    }

    /// Returns a human readable string that depends on one or more variables.
    pub fn detail_string(&self) -> String {
        match self.action {
            HistoryAction::NovelAdd => fl!("added-novel"),
            HistoryAction::NovelDelete => fl!("deleted-novel"),
            HistoryAction::NovelUpdate => fl!("updated-novel"),
            HistoryAction::NovelListChange => {
                if let Some(list_status) = self.list_status {
                    format!("{} {}", &fl!("moved-novel"), list_status.to_string())
                } else {
                    "".to_string()
                }
            }
            HistoryAction::ContentRead => {
                if let Some(content) = self.content.as_ref() {
                    let mut content_string = String::new();
                    if content.volumes > 0 {
                        content_string.push_str(&format!("{} {} ", fl!("volume"), content.volumes));
                    }
                    if content.chapters > 0.0 {
                        if let Some(title) = &self.named_chapter {
                            content_string.push_str(&format!(
                                "{} {} - {} ",
                                fl!("chapter"),
                                content.chapters,
                                title
                            ));
                        } else {
                            content_string.push_str(&format!(
                                "{} {} ",
                                fl!("chapter"),
                                content.chapters
                            ));
                        }
                    }
                    if content.side_stories > 0 {
                        content_string.push_str(&format!(
                            "{} {} ",
                            fl!("side-story"),
                            content.side_stories
                        ));
                    }
                    return content_string;
                }

                "?".to_string()
            }
        }
    }

    pub fn time_string(&self) -> String {
        let dt = Utc.timestamp_millis(self.time);
        return dt.format("%d %B %Y, %H:%M:%S").to_string();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NovelHistory {
    pub items: Vec<NovelHistoryItem>,
}

impl Default for NovelHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl NovelHistory {
    pub fn new() -> Self {
        NovelHistory { items: vec![] }
    }

    /// Find the latest read action from history and return it if found.
    pub fn find_last_read(&self) -> Option<NovelHistoryItem> {
        let mut vec = self.items.clone();
        vec.sort_by(|a, b| b.time.cmp(&a.time));
        for item in vec.iter() {
            if item.action == HistoryAction::ContentRead {
                return Some(item.clone());
            }
        }

        None
    }

    /// Use chapter title to find a `NovelHistoryItem`.
    pub fn find_chapter_title(&self, chapter_title: &str) -> Option<NovelHistoryItem> {
        let vec = self.items.clone();
        for item in vec.iter() {
            if item.action == HistoryAction::ContentRead {
                if let Some(chapter) = &item.named_chapter {
                    if chapter == chapter_title {
                        return Some(item.clone());
                    }
                }
            }
        }

        None
    }

    /// Write the history data into a file.
    pub fn write_to_file(&self) -> Result<(), ErisError> {
        debug!("history:write_to_file");
        let path = &data_dir(HISTORY_FILE);
        let f = File::create(&path).unwrap();
        let writer = BufWriter::new(f);

        serialize_into(writer, self).context(ErisError::WriteToDisk)?;

        Ok(())
    }

    /// Try to read the history data from a file.
    pub fn open() -> Result<Self, ErisError> {
        let path = &data_dir(HISTORY_FILE);
        if path.as_path().exists() {
            let f = File::open(&path).context(ErisError::ReadFromDisk)?;
            let reader = BufReader::new(&f);

            if let Ok(history) = deserialize_from(reader).context(ErisError::Unknown) {
                return Ok(history);
            }
        }

        Ok(NovelHistory::default())
    }
}

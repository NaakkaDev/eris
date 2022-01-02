use crate::utils::data_dir;
use crate::DATA_DIR;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GeneralSettings {
    pub show_sidebar: bool,
    pub data_dir: PathBuf,
    pub mouse_2_action: NovelListAction,
    pub mouse_3_action: NovelListAction,
    pub mouse_4_action: NovelListAction,
    pub mouse_5_action: NovelListAction,
    pub reader: Option<PathBuf>,
    pub reader_args: String,
    pub language: Option<String>,
    pub open_with_windows: bool,
    pub start_minimized: bool,
    pub check_update: bool,
    pub window_state_enabled: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl GeneralSettings {
    pub fn new() -> GeneralSettings {
        GeneralSettings {
            show_sidebar: true,
            data_dir: data_dir(DATA_DIR),
            mouse_2_action: NovelListAction::Read,
            mouse_3_action: NovelListAction::OpenContextMenu,
            mouse_4_action: NovelListAction::DecreaseChapterCount,
            mouse_5_action: NovelListAction::IncreaseChapterCount,
            reader: None,
            reader_args: "-f %f -p %p".to_string(),
            language: None,
            open_with_windows: false,
            start_minimized: false,
            check_update: false,
            window_state_enabled: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[repr(i32)]
pub enum NovelListAction {
    Nothing = 0,
    OpenNovelInfo,
    EditNovelInfo,
    Read,
    IncreaseChapterCount,
    DecreaseChapterCount,
    OpenContextMenu,
}

impl ToString for NovelListAction {
    fn to_string(&self) -> String {
        match self {
            NovelListAction::Nothing => fl!("action-nothing"),
            NovelListAction::OpenNovelInfo => fl!("action-open-novel-info"),
            NovelListAction::EditNovelInfo => fl!("action-edit-novel-info"),
            NovelListAction::Read => fl!("action-read"),
            NovelListAction::IncreaseChapterCount => fl!("action-increase-chapter-count"),
            NovelListAction::DecreaseChapterCount => fl!("action-decrease-chapter-count"),
            NovelListAction::OpenContextMenu => fl!("action-open-context-menu"),
        }
    }
}

impl NovelListAction {
    pub fn to_i32(&self) -> i32 {
        self.to_owned() as i32
    }

    pub fn vec() -> Vec<String> {
        vec![
            NovelListAction::Nothing.to_string(),
            NovelListAction::OpenNovelInfo.to_string(),
            NovelListAction::EditNovelInfo.to_string(),
            NovelListAction::Read.to_string(),
            NovelListAction::IncreaseChapterCount.to_string(),
            NovelListAction::DecreaseChapterCount.to_string(),
            NovelListAction::OpenContextMenu.to_string(),
        ]
    }

    pub fn from_i32(value: i32) -> NovelListAction {
        match value {
            0 => NovelListAction::Nothing,
            1 => NovelListAction::OpenNovelInfo,
            2 => NovelListAction::EditNovelInfo,
            3 => NovelListAction::Read,
            4 => NovelListAction::IncreaseChapterCount,
            5 => NovelListAction::DecreaseChapterCount,
            6 => NovelListAction::OpenContextMenu,
            _ => NovelListAction::Nothing,
        }
    }
}

use crate::ui::novel_list::Column;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListSettings {
    /// Sorting order for each list.
    pub list_sort_order: [Sorting; 6],
    /// `HashMap` with `<column id, column width>` values for each list.
    pub column_width: [HashMap<i32, i32>; 6],
    /// Vector of booleans that decide if the corresponding `Column` is visible or not.
    pub visible_columns: Vec<bool>,
    /// Which tab to display when opening novel dialog.
    pub open_info_behavior: i32,
    /// Always display the selected tab instead of the one that was previously open.
    pub always_open_selected_tab: bool,
}

impl ListSettings {
    pub fn new() -> Self {
        ListSettings {
            list_sort_order: [Sorting::default(); 6],
            // HashMap has no COPY so..
            column_width: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            visible_columns: vec![
                false, // 0 (id)
                true,  // 1 (status)
                false, // 2
                false, // 3 (cco)
                true,  // 4
                true,  // 5 (chapters)
                true,  // 6 (side stories)
                false, // 7 (volumes)
                true,  // 8
                false, // 9
                true,  // 10
            ],
            open_info_behavior: 0,
            always_open_selected_tab: false,
        }
    }
}

impl Default for ListSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
pub struct Sorting {
    pub column_id: u32,
    pub is_sort_indicator: bool,
    pub sort_order: i8,
}

impl Default for Sorting {
    fn default() -> Self {
        Sorting {
            column_id: Column::LastRead as u32,
            is_sort_indicator: true,
            sort_order: 0,
        }
    }
}

impl Sorting {
    pub fn to_gtk_sort_type(&self) -> gtk::SortType {
        match self.sort_order {
            1 => gtk::SortType::Ascending,
            _ => gtk::SortType::Descending,
        }
    }
}

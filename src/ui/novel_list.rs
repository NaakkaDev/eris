use crate::app::novel::Novel;

use crate::app::settings::{NovelListAction, Settings, Sorting};
use crate::app::AppRuntime;
use crate::appop::messages::SortingMessage;
use crate::utils::gtk::BuilderExtManualCustom;
use chrono::NaiveDateTime;
use gdk::{DragAction, EventButton, ModifierType};
use gio::prelude::*;
use glib::{SignalHandlerId, Type};
use gtk::prelude::GtkMenuExt;
use gtk::{prelude::*, Adjustment, DestDefaults, SortColumn, TreePath, TreeView};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub const LIST_COUNT: i32 = 5;
pub const COLUMN_COUNT: usize = 11;

pub const ID_COLUMN: i32 = 0;
pub const COLUMN_TYPES: [Type; COLUMN_COUNT] = [
    Type::STRING, // id
    Type::STRING, // status
    Type::OBJECT, // status icon
    Type::STRING,
    Type::STRING, // title
    Type::STRING,
    Type::STRING,
    Type::I32,
    Type::STRING,
    Type::STRING,
    Type::STRING,
];

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[repr(i32)]
pub enum ListStatus {
    Reading = 0,
    PlanToRead,
    OnHold,
    Completed,
    Dropped,
}

impl Default for ListStatus {
    fn default() -> Self {
        ListStatus::PlanToRead
    }
}

impl ToString for ListStatus {
    fn to_string(&self) -> String {
        match self {
            ListStatus::Reading => fl!("reading"),
            ListStatus::PlanToRead => fl!("plan-to-read"),
            ListStatus::OnHold => fl!("on-hold"),
            ListStatus::Completed => fl!("completed"),
            ListStatus::Dropped => fl!("dropped"),
        }
    }
}

impl ListStatus {
    pub fn vec() -> Vec<String> {
        vec![
            ListStatus::Reading.to_string(),
            ListStatus::PlanToRead.to_string(),
            ListStatus::OnHold.to_string(),
            ListStatus::Completed.to_string(),
            ListStatus::Dropped.to_string(),
        ]
    }

    pub fn from_i32(i: i32) -> ListStatus {
        match i {
            0 => ListStatus::Reading,
            1 => ListStatus::PlanToRead,
            2 => ListStatus::OnHold,
            3 => ListStatus::Completed,
            4 => ListStatus::Dropped,
            _ => ListStatus::Reading,
        }
    }

    pub fn to_i32(self) -> i32 {
        self.to_owned() as i32
    }

    pub(crate) fn combo_box_id(&self) -> &str {
        match *self {
            ListStatus::Reading => "0",
            ListStatus::PlanToRead => "1",
            ListStatus::OnHold => "2",
            ListStatus::Completed => "3",
            ListStatus::Dropped => "4",
        }
    }

    pub(crate) fn from_combo_box_id(id: &str) -> ListStatus {
        match id {
            "0" => ListStatus::Reading,
            "1" => ListStatus::PlanToRead,
            "2" => ListStatus::OnHold,
            "3" => ListStatus::Completed,
            "4" => ListStatus::Dropped,
            _ => ListStatus::Reading,
        }
    }

    pub fn from_name(name: &str) -> ListStatus {
        match name {
            "reading" => ListStatus::Reading,
            "plan_to_read" => ListStatus::PlanToRead,
            "on_hold" => ListStatus::OnHold,
            "completed" => ListStatus::Completed,
            "dropped" => ListStatus::Dropped,
            _ => ListStatus::Reading,
        }
    }

    pub fn treeview_id(&self) -> &'static str {
        match *self {
            ListStatus::Reading => "TreeView1",
            ListStatus::PlanToRead => "TreeView2",
            ListStatus::OnHold => "TreeView3",
            ListStatus::Completed => "TreeView4",
            ListStatus::Dropped => "TreeView5",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(i32)]
pub enum Column {
    Status = 1,
    StatusIcon,
    OriginalLanguage,
    Title,
    ChaptersRead,
    SideStoriesRead,
    VolumesRead,
    ChaptersAvailable,
    Score,
    LastUpdate,
}

impl Column {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => Column::Status,
            2 => Column::StatusIcon,
            3 => Column::OriginalLanguage,
            4 => Column::Title,
            5 => Column::ChaptersRead,
            6 => Column::SideStoriesRead,
            7 => Column::VolumesRead,
            8 => Column::ChaptersAvailable,
            9 => Column::Score,
            10 => Column::LastUpdate,
            _ => Column::Title,
        }
    }
}

#[derive(Debug)]
pub struct Trees {
    pub reading: gtk::TreeView,
    pub plan_to_read: gtk::TreeView,
    pub on_hold: gtk::TreeView,
    pub completed: gtk::TreeView,
    pub dropped: gtk::TreeView,
}

impl Trees {
    fn new(builder: &gtk::Builder) -> Self {
        Trees {
            reading: builder.get::<gtk::TreeView>("TreeView1"),
            plan_to_read: builder.get::<gtk::TreeView>("TreeView2"),
            on_hold: builder.get::<gtk::TreeView>("TreeView3"),
            completed: builder.get::<gtk::TreeView>("TreeView4"),
            dropped: builder.get::<gtk::TreeView>("TreeView5"),
        }
    }
}

impl Index<i32> for Trees {
    type Output = gtk::TreeView;
    fn index(&self, i: i32) -> &gtk::TreeView {
        match i {
            0 => &self.reading,
            1 => &self.plan_to_read,
            2 => &self.on_hold,
            3 => &self.completed,
            4 => &self.dropped,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl IndexMut<i32> for Trees {
    fn index_mut(&mut self, i: i32) -> &mut gtk::TreeView {
        match i {
            0 => &mut self.reading,
            1 => &mut self.plan_to_read,
            2 => &mut self.on_hold,
            3 => &mut self.completed,
            4 => &mut self.dropped,
            _ => panic!("unknown field: {}", i),
        }
    }
}

#[derive(Debug)]
pub struct Lists {
    pub reading: gtk::ListStore,
    pub plan_to_read: gtk::ListStore,
    pub on_hold: gtk::ListStore,
    pub completed: gtk::ListStore,
    pub dropped: gtk::ListStore,
}

impl Default for Lists {
    fn default() -> Self {
        Lists {
            reading: gtk::ListStore::new(&COLUMN_TYPES),
            plan_to_read: gtk::ListStore::new(&COLUMN_TYPES),
            on_hold: gtk::ListStore::new(&COLUMN_TYPES),
            completed: gtk::ListStore::new(&COLUMN_TYPES),
            dropped: gtk::ListStore::new(&COLUMN_TYPES),
        }
    }
}

impl Index<i32> for Lists {
    type Output = gtk::ListStore;
    fn index(&self, i: i32) -> &gtk::ListStore {
        match i {
            0 => &self.reading,
            1 => &self.plan_to_read,
            2 => &self.on_hold,
            3 => &self.completed,
            4 => &self.dropped,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl IndexMut<i32> for Lists {
    fn index_mut(&mut self, i: i32) -> &mut gtk::ListStore {
        match i {
            0 => &mut self.reading,
            1 => &mut self.plan_to_read,
            2 => &mut self.on_hold,
            3 => &mut self.completed,
            4 => &mut self.dropped,
            _ => panic!("unknown field: {}", i),
        }
    }
}

impl Lists {
    pub fn new() -> Self {
        let lists = Lists::default();

        // Set sort function for datetime (last update) column
        let last_update_column_id = Column::LastUpdate as u32;
        lists
            .reading
            .set_sort_func(SortColumn::Index(last_update_column_id), list_sort_datetime);
        lists
            .plan_to_read
            .set_sort_func(SortColumn::Index(last_update_column_id), list_sort_datetime);
        lists
            .on_hold
            .set_sort_func(SortColumn::Index(last_update_column_id), list_sort_datetime);
        lists
            .completed
            .set_sort_func(SortColumn::Index(last_update_column_id), list_sort_datetime);
        lists
            .dropped
            .set_sort_func(SortColumn::Index(last_update_column_id), list_sort_datetime);

        lists
    }
}

#[derive(Debug)]
pub struct NovelList {
    pub trees: Trees,
    pub lists: Lists,

    pub label_reading: gtk::Label,
    pub label_plan_to_read: gtk::Label,
    pub label_on_hold: gtk::Label,
    pub label_completed: gtk::Label,
    pub label_dropped: gtk::Label,

    pub counts: [i32; 5],

    pub active_page: i32,
    pub active_list: ListStatus,
    pub active_iter: Option<gtk::TreeIter>,
    pub active_novel: Option<Novel>,

    pub connect_button_handlers: HashMap<i32, SignalHandlerId>,
    pub tooltip_handler: HashMap<i32, SignalHandlerId>,
}

impl NovelList {
    pub fn new(builder: &gtk::Builder) -> NovelList {
        let trees = Trees::new(builder);
        let lists = Lists::new();

        for i in 0..LIST_COUNT {
            let treeview = trees.index(i);

            if i == 5 {
            } else {
                // Set corresponding `ListStore` to the correct treeview model
                treeview.set_model(Some(lists.index(i)));
            }
        }

        let label_reading = builder.get::<gtk::Label>("NotebookLabel1");
        let label_plan_to_read = builder.get::<gtk::Label>("NotebookLabel2");
        let label_on_hold = builder.get::<gtk::Label>("NotebookLabel3");
        let label_completed = builder.get::<gtk::Label>("NotebookLabel4");
        let label_dropped = builder.get::<gtk::Label>("NotebookLabel5");

        // Translate
        label_reading.set_label(&fl!("reading"));
        label_plan_to_read.set_label(&fl!("plan-to-read"));
        label_on_hold.set_label(&fl!("on-hold"));
        label_completed.set_label(&fl!("completed"));
        label_dropped.set_label(&fl!("dropped"));

        NovelList {
            trees,
            lists,

            label_reading,
            label_plan_to_read,
            label_on_hold,
            label_completed,
            label_dropped,

            counts: [0; 5],

            active_page: 0,
            active_list: ListStatus::Reading,
            active_iter: None,
            active_novel: None,

            connect_button_handlers: HashMap::new(),
            tooltip_handler: HashMap::new(),
        }
    }

    /// This is ran only once on app startup.
    pub fn connect(&mut self, builder: &gtk::Builder, app_runtime: AppRuntime, settings: &Settings) {
        // Set correct sorting for each list
        self.lists.reading.set_sort_column_id(
            SortColumn::Index(settings.list.list_sort_order[0].column_id),
            settings.list.list_sort_order[0].to_gtk_sort_type(),
        );
        self.lists.plan_to_read.set_sort_column_id(
            SortColumn::Index(settings.list.list_sort_order[1].column_id),
            settings.list.list_sort_order[1].to_gtk_sort_type(),
        );
        self.lists.on_hold.set_sort_column_id(
            SortColumn::Index(settings.list.list_sort_order[2].column_id),
            settings.list.list_sort_order[2].to_gtk_sort_type(),
        );
        self.lists.completed.set_sort_column_id(
            SortColumn::Index(settings.list.list_sort_order[3].column_id),
            settings.list.list_sort_order[3].to_gtk_sort_type(),
        );
        self.lists.dropped.set_sort_column_id(
            SortColumn::Index(settings.list.list_sort_order[4].column_id),
            settings.list.list_sort_order[4].to_gtk_sort_type(),
        );

        for i in 0..LIST_COUNT {
            let treeview = self.trees.index(i);

            let list = match ListStatus::from_i32(i) {
                ListStatus::Reading => &self.lists.reading,
                ListStatus::PlanToRead => &self.lists.plan_to_read,
                ListStatus::OnHold => &self.lists.on_hold,
                ListStatus::Completed => &self.lists.completed,
                ListStatus::Dropped => &self.lists.dropped,
            };

            // What happens when list row is activated
            treeview.connect_row_activated(
                glib::clone!(@strong app_runtime, @strong i, @strong list => move |tree, path, _col| {
                    action_open_novel_dialog(&app_runtime, tree, path, i, false);
                }),
            );

            // ===============================
            // Drag and drop individual novels from one list to another,
            // supports dragging only one item at the time.
            //
            let targets = vec![gtk::TargetEntry::new("UTF8_STRING", gtk::TargetFlags::SAME_APP, 0)];

            treeview.drag_source_set(ModifierType::MODIFIER_MASK, &targets, DragAction::MOVE);
            treeview.connect_drag_data_get(glib::clone!(@strong app_runtime => move |tree, _, selection, _, _| {
                let model = tree.model().unwrap();
                let (path, _) = tree.cursor();
                let iter = model.iter(&path.unwrap()).unwrap();
                let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
                selection.set_text(&novel_id);

                app_runtime.update_state_with(move |state| {
                    let novel = state.get_by_id(novel_id).unwrap();
                    state.ui.lists.active_list = novel.settings.list_status;
                    state.ui.lists.active_iter = Some(iter);
                });
            }));

            let tab = builder.get::<gtk::Label>(&format!("NotebookLabel{}", i + 1));
            tab.drag_dest_set(DestDefaults::ALL, &targets, DragAction::MOVE);
            tab.connect_drag_data_received(glib::clone!(@strong app_runtime => move |tab, _, _, _, data, _, _| {
                if let Some(novel_id) = data.text() {
                    let to_list = ListStatus::from_name(tab.widget_name().as_str());
                    app_runtime.update_state_with(move |state| {
                        // Do nothing if moving to the same list
                        if state.ui.lists.active_list == to_list {
                            return;
                        }

                        state.move_novel(novel_id.to_string(), to_list)
                    });
                }
            }));
            // Drag and drop end
            // ===============================
        }
    }

    /// This will run more than one time.
    pub fn connect_mouse_actions(&mut self, app_runtime: AppRuntime, settings: &Settings) {
        if !self.connect_button_handlers.is_empty() {
            for (i, handler) in self.connect_button_handlers.drain() {
                // If the vector that stores the signal handler ids is not empty
                // then loop through it for each `TreeView` and disconnect button press events.
                self.trees.index(i).disconnect(handler);
            }
        }

        for i in 0..LIST_COUNT {
            let treeview = self.trees.index(i);

            let list = match ListStatus::from_i32(i) {
                ListStatus::Reading => &self.lists.reading,
                ListStatus::PlanToRead => &self.lists.plan_to_read,
                ListStatus::OnHold => &self.lists.on_hold,
                ListStatus::Completed => &self.lists.completed,
                ListStatus::Dropped => &self.lists.dropped,
            };

            let handler = treeview.connect_button_press_event(
                glib::clone!(@strong app_runtime, @strong settings, @strong i, @strong list => move |tree, event| {
                    set_list_actions(&app_runtime, event, &settings, tree, i);

                    gtk::Inhibit(false)
                }),
            );

            // Add the handler into the hashmap with the treeview id
            // so it can be disconnected later if needed.
            self.connect_button_handlers.insert(i, handler);
        }
    }

    pub fn connect_tooltips(&mut self, settings: &Settings) {
        // Disconnect any stale-to-be tooltip events
        if !self.tooltip_handler.is_empty() {
            for (i, handler) in self.tooltip_handler.drain() {
                // If the vector that stores the signal handler ids is not empty
                // then loop through it for each `TreeView` and disconnect tooltip events.
                self.trees.index(i).disconnect(handler);
            }
        }

        // Only the status column has tooltips so if they are not visible then do nothing else
        if !settings.list.visible_columns[Column::Status as usize] {
            return;
        }

        for i in 0..LIST_COUNT {
            let treeview = self.trees.index(i);

            let handler = treeview.connect_query_tooltip(|tree, _x, y, _keyboard, tooltip| {
                let model = tree.model().unwrap();
                let (path, _) = tree.cursor();
                if let Some(path) = path {
                    // If the mouse is past the rows then do not show the tooltip
                    if y > (model.iter_n_children(None) * 19) + 26 {
                        return false;
                    }

                    // -1 because logic.. guess it's the first element in the list of (visible) columns
                    let column = tree.column(Column::Status as i32 - 1).unwrap();
                    let cell = column.cells().get(0).unwrap().to_owned();

                    let iter = model.iter(&path).unwrap();
                    let novel_status = model.value(&iter, Column::Status as i32).get::<String>().unwrap();

                    tooltip.set_text(Some(&novel_status));
                    tree.set_tooltip_cell(tooltip, None, Some(&column), Some(&cell));

                    return true;
                }

                false
            });

            self.tooltip_handler.insert(i, handler);
        }
    }

    pub fn add_columns(&mut self, app_runtime: AppRuntime, settings: &Settings) {
        for i in 0..LIST_COUNT {
            let treeview = self.trees.index(i);

            // Remove any existing columns
            for col in treeview.columns() {
                treeview.remove_column(&col);
            }

            // Add columns to the treeviews
            add_columns(i, treeview, app_runtime.clone(), settings);
        }

        self.connect_tooltips(settings);
    }

    pub fn populate_columns(&mut self, novel_data: &[Novel]) {
        for (_d_index, novel) in novel_data.iter().enumerate() {
            let values: [(u32, &dyn ToValue); COLUMN_COUNT] = [
                (ID_COLUMN as u32, &novel.id),
                (Column::Status as u32, &novel.status.to_string()),
                (Column::StatusIcon as u32, &novel.status_pix()),
                (Column::OriginalLanguage as u32, &novel.orig_lang()),
                (Column::Title as u32, &novel.title()),
                (Column::ChaptersRead as u32, &novel.chapters_read_str()),
                (
                    Column::SideStoriesRead as u32,
                    &novel.settings.content_read.side_stories,
                ),
                (Column::VolumesRead as u32, &novel.settings.content_read.volumes),
                (Column::ChaptersAvailable as u32, &novel.content()),
                (Column::Score as u32, &novel.settings.score),
                (Column::LastUpdate as u32, &novel.settings.last_updated_string()),
            ];

            self.counts[novel.settings.list_status.to_i32() as usize] =
                self.counts[novel.settings.list_status.to_i32() as usize] + 1;

            match &novel.settings.list_status {
                ListStatus::Reading => self.lists.reading.set(&self.lists.reading.append(), &values),
                ListStatus::PlanToRead => self.lists.plan_to_read.set(&self.lists.plan_to_read.append(), &values),
                ListStatus::OnHold => self.lists.on_hold.set(&self.lists.on_hold.append(), &values),
                ListStatus::Completed => self.lists.completed.set(&self.lists.completed.append(), &values),
                ListStatus::Dropped => self.lists.dropped.set(&self.lists.dropped.append(), &values),
            }
        }

        for i in 0..LIST_COUNT {
            match i {
                0 => self.label_reading.set_text(&format!(
                    "{} ({})",
                    ListStatus::from_i32(i).to_string(),
                    self.counts[i as usize]
                )),
                1 => self.label_plan_to_read.set_text(&format!(
                    "{} ({})",
                    ListStatus::from_i32(i).to_string(),
                    self.counts[i as usize]
                )),
                2 => self.label_on_hold.set_text(&format!(
                    "{} ({})",
                    ListStatus::from_i32(i).to_string(),
                    self.counts[i as usize]
                )),
                3 => self.label_completed.set_text(&format!(
                    "{} ({})",
                    ListStatus::from_i32(i).to_string(),
                    self.counts[i as usize]
                )),
                4 => self.label_dropped.set_text(&format!(
                    "{} ({})",
                    ListStatus::from_i32(i).to_string(),
                    self.counts[i as usize]
                )),
                _ => {}
            }
        }
    }

    pub fn list_insert(&mut self, novel: &Novel) {
        let values: [(u32, &dyn ToValue); COLUMN_COUNT] = [
            (ID_COLUMN as u32, &novel.id),
            (Column::Status as u32, &novel.status.to_string()),
            (Column::StatusIcon as u32, &novel.status_pix()),
            (Column::OriginalLanguage as u32, &novel.orig_lang()),
            (Column::Title as u32, &novel.title()),
            (Column::ChaptersRead as u32, &novel.chapters_read_str()),
            (
                Column::SideStoriesRead as u32,
                &novel.settings.content_read.side_stories,
            ),
            (Column::VolumesRead as u32, &novel.settings.content_read.volumes),
            (Column::ChaptersAvailable as u32, &novel.content()),
            (Column::Score as u32, &novel.settings.score),
            (Column::LastUpdate as u32, &novel.settings.last_updated_string()),
        ];

        self.counts[novel.settings.list_status.to_i32() as usize] =
            self.counts[novel.settings.list_status.to_i32() as usize] + 1;

        // Insert new row and update the tab label count
        match &novel.settings.list_status {
            ListStatus::Reading => {
                self.lists.reading.insert_with_values(Some(0), &values);
                self.label_reading
                    .set_text(&format!("{} ({})", ListStatus::Reading.to_string(), self.counts[0]));
            }
            ListStatus::PlanToRead => {
                self.lists.plan_to_read.insert_with_values(Some(0), &values);
                self.label_plan_to_read.set_text(&format!(
                    "{} ({})",
                    ListStatus::PlanToRead.to_string(),
                    self.counts[1]
                ));
            }
            ListStatus::OnHold => {
                self.lists.on_hold.insert_with_values(Some(0), &values);
                self.label_on_hold
                    .set_text(&format!("{} ({})", ListStatus::OnHold.to_string(), self.counts[2]));
            }
            ListStatus::Completed => {
                self.lists.completed.insert_with_values(Some(0), &values);
                self.label_completed
                    .set_text(&format!("{} ({})", ListStatus::Completed.to_string(), self.counts[3]));
            }
            ListStatus::Dropped => {
                self.lists.dropped.insert_with_values(Some(0), &values);
                self.label_dropped
                    .set_text(&format!("{} ({})", ListStatus::Dropped.to_string(), self.counts[4]));
            }
        }

        self.scroll_to_top(novel.settings.list_status);
    }

    pub fn list_remove(&mut self, novel: &Novel, old_iter: Option<gtk::TreeIter>) {
        // Get the correct `ListStore` from the novel list status
        // Use `active_list` if `old_iter` is present as that means
        // that the novel will be moved
        let list_status = if old_iter.is_some() {
            self.active_list
        } else {
            novel.settings.list_status
        };
        let list = match list_status {
            ListStatus::Reading => &self.lists.reading,
            ListStatus::PlanToRead => &self.lists.plan_to_read,
            ListStatus::OnHold => &self.lists.on_hold,
            ListStatus::Completed => &self.lists.completed,
            ListStatus::Dropped => &self.lists.dropped,
        };

        self.counts[list_status.to_i32() as usize] = self.counts[list_status.to_i32() as usize] - 1;

        // Update the novel the tab label count
        match list_status {
            ListStatus::Reading => {
                self.label_reading
                    .set_text(&format!("{} ({})", ListStatus::Reading.to_string(), self.counts[0]));
            }
            ListStatus::PlanToRead => {
                self.label_plan_to_read.set_text(&format!(
                    "{} ({})",
                    ListStatus::PlanToRead.to_string(),
                    self.counts[1]
                ));
            }
            ListStatus::OnHold => {
                self.label_on_hold
                    .set_text(&format!("{} ({})", ListStatus::OnHold.to_string(), self.counts[2]));
            }
            ListStatus::Completed => {
                self.label_completed
                    .set_text(&format!("{} ({})", ListStatus::Completed.to_string(), self.counts[3]));
            }
            ListStatus::Dropped => {
                self.label_dropped
                    .set_text(&format!("{} ({})", ListStatus::Dropped.to_string(), self.counts[4]));
            }
        }

        let iter = if let Some(iter) = old_iter {
            iter
        } else if let Some(iter) = self.find_iter(novel) {
            iter
        } else {
            error!("Cannot remove novel {} from list, iter not found.", novel.id);
            return;
        };

        list.remove(&iter);

        self.scroll_to_top(list_status);
    }

    /// Move novel from list A to list B
    pub fn list_move(&mut self, novel: &Novel, old_iter: Option<gtk::TreeIter>) {
        // Move novel to another list
        // Remove from old list
        // then insert into the new one
        self.list_remove(novel, old_iter);
        self.list_insert(novel);
    }

    /// Update the lists
    pub fn list_update(&mut self, novel: &Novel) {
        let tree_iter = self.find_iter(novel);
        if let Some(iter) = &tree_iter {
            // Decide which list to update
            let list = match novel.settings.list_status {
                ListStatus::Reading => &self.lists.reading,
                ListStatus::PlanToRead => &self.lists.plan_to_read,
                ListStatus::OnHold => &self.lists.on_hold,
                ListStatus::Completed => &self.lists.completed,
                ListStatus::Dropped => &self.lists.dropped,
            };

            // Updated the columns on the selected row
            list.set_value(iter, Column::Status as u32, &novel.status.to_str().to_value());
            list.set_value(iter, Column::StatusIcon as u32, &novel.status_pix().to_value());
            list.set_value(iter, Column::OriginalLanguage as u32, &novel.orig_lang().to_value());
            list.set_value(iter, Column::Title as u32, &novel.title().to_value());
            list.set_value(iter, Column::ChaptersRead as u32, &novel.chapters_read_str().to_value());
            list.set_value(
                iter,
                Column::SideStoriesRead as u32,
                &novel.settings.content_read.side_stories.to_value(),
            );
            list.set_value(
                iter,
                Column::VolumesRead as u32,
                &novel.settings.content_read.volumes.to_value(),
            );
            list.set_value(iter, Column::ChaptersAvailable as u32, &novel.content().to_value());
            list.set_value(iter, Column::Score as u32, &novel.settings.score.to_value());
            list.set_value(
                iter,
                Column::LastUpdate as u32,
                &novel.settings.last_updated_string().to_value(),
            );
        }

        self.scroll_to_top(novel.settings.list_status);
    }

    /// Find correct `gtk::TreeIter` for the novel.
    pub fn find_iter(&self, novel: &Novel) -> Option<gtk::TreeIter> {
        let model = self.trees.index(novel.settings.list_status.to_i32()).model().unwrap();

        for i in 0..(model.iter_n_children(None)) {
            // Check if this is the correct iter
            let iter = model.iter_from_string(&i.to_string());
            if let Some(iter) = iter {
                // If the novel id in the row matches with the novel id being read then the correct one was found.
                let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
                if novel_id == novel.id {
                    return Some(iter);
                }
            }
        }

        None
    }

    /// Does what is says.
    pub fn scroll_to_top(&self, list: ListStatus) {
        // First move to scrollbar down a bit, because wonky
        self.trees[list.to_i32()].vadjustment().unwrap().set_value(35.0);
        // then move it to top
        self.trees[list.to_i32()].vadjustment().unwrap().set_value(0.1);
    }
}

fn list_actions(
    app_runtime: &AppRuntime,
    event: &EventButton,
    tree: &TreeView,
    list_action: &NovelListAction,
    treeview_index: i32,
) {
    match list_action {
        NovelListAction::OpenNovelInfo => {
            let (path, _) = tree.cursor();
            if let Some(path) = &path {
                action_open_novel_dialog(app_runtime, tree, path, treeview_index, false);
            }
        }
        NovelListAction::EditNovelInfo => {
            let (path, _) = tree.cursor();
            if let Some(path) = &path {
                action_open_novel_dialog(app_runtime, tree, path, treeview_index, true);
            }
        }
        NovelListAction::ReadNext => {
            action_read_novel(app_runtime, tree);
        }
        NovelListAction::IncreaseChapterCount => {
            action_increase_read_count(app_runtime, tree);
        }
        NovelListAction::DecreaseChapterCount => {
            action_decreace_read_count(app_runtime, tree);
        }
        NovelListAction::OpenContextMenu => {
            action_open_menu(app_runtime, tree, treeview_index, event);
        }
        _ => {}
    }
}

/// Sort function for novel lists which sorts the `Column::LastUpdate` column by turning
/// the datetime string into a timemstamp and then comparing the values.
pub fn list_sort_datetime(model: &gtk::TreeModel, a_iter: &gtk::TreeIter, b_iter: &gtk::TreeIter) -> Ordering {
    let date_a = model.value(a_iter, Column::LastUpdate as i32).get::<String>().unwrap();
    let date_b = model.value(b_iter, Column::LastUpdate as i32).get::<String>().unwrap();

    let dt_a = NaiveDateTime::parse_from_str(&date_a, "%d %B %Y, %H:%M:%S").unwrap();
    let dt_b = NaiveDateTime::parse_from_str(&date_b, "%d %B %Y, %H:%M:%S").unwrap();

    dt_a.cmp(&dt_b)
}

pub fn set_list_actions(
    app_runtime: &AppRuntime,
    event: &EventButton,
    settings: &Settings,
    tree: &TreeView,
    treeview_index: i32,
) {
    match event.button() {
        2 => list_actions(
            app_runtime,
            event,
            tree,
            &settings.general.mouse_2_action,
            treeview_index,
        ),
        3 => list_actions(
            app_runtime,
            event,
            tree,
            &settings.general.mouse_3_action,
            treeview_index,
        ),
        4 => list_actions(
            app_runtime,
            event,
            tree,
            &settings.general.mouse_4_action,
            treeview_index,
        ),
        5 => list_actions(
            app_runtime,
            event,
            tree,
            &settings.general.mouse_5_action,
            treeview_index,
        ),
        _ => {}
    }
}

pub fn action_read_novel(app_runtime: &AppRuntime, tree: &TreeView) {
    let model = tree.model().unwrap();
    let (path, _) = tree.cursor();
    let iter = model.iter(&path.unwrap()).unwrap();
    let text = model.value(&iter, ID_COLUMN).get::<String>().unwrap();

    app_runtime.update_state_with(move |state| {
        if let Some(novel) = state.get_by_id(text) {
            state.read_novel(novel);
        }
    });
}

pub fn action_open_menu(app_runtime: &AppRuntime, tree: &TreeView, treeview_index: i32, event: &gdk::EventButton) {
    let model = tree.model().unwrap();
    let (path, _) = tree.cursor();
    if path.is_none() {
        return;
    }
    let iter = model.iter(&path.unwrap()).unwrap();
    let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
    let menu = gtk::Menu::new();
    let menuitem_open = gtk::MenuItem::new();
    menuitem_open.set_label(&fl!("menu-open-edit"));
    menuitem_open.connect_activate(
        glib::clone!(@strong app_runtime, @strong novel_id, @strong treeview_index, @strong iter => move |_| {
            let iter = iter;
            let novel_id = novel_id.clone();
            app_runtime.update_state_with(move |mut state| {
                if let Some(novel) = state.get_by_id(novel_id) {
                    state.ui.lists.active_list = ListStatus::from_i32(treeview_index);
                    state.ui.lists.active_iter = Some(iter);
                    state.ui.lists.active_novel = Some(novel.clone());
                    state.ui.show_novel_dialog(&novel, &state.settings.read());
                }
            });
        }),
    );
    let menuitem_read = gtk::MenuItem::new();
    menuitem_read.set_label(&fl!("action-read-next"));
    menuitem_read.connect_activate(glib::clone!(@strong app_runtime, @strong novel_id => move |_| {
        let novel_id = novel_id.clone();
        app_runtime.update_state_with(move |state| {
            if let Some(novel) = state.get_by_id(novel_id) {
                state.read_novel(novel);
            }
        });
    }));
    let menuitem_slug = gtk::MenuItem::new();
    menuitem_slug.set_label(&fl!("menu-open-webpage"));
    menuitem_slug.connect_activate(glib::clone!(@strong app_runtime, @strong novel_id => move |_| {
        let novel_id = novel_id.clone();
        app_runtime.update_state_with(move |state| {
            if let Some(novel) = state.get_by_id(novel_id) {
                novel.open_slug();
            }
        });
    }));

    menu.set_attach_widget(Some(tree));
    menu.add(&menuitem_open);
    menu.add(&menuitem_read);
    menu.add(&menuitem_slug);
    menu.show_all();
    menu.popup_at_pointer(Some(event));
}

pub fn action_decreace_read_count(app_runtime: &AppRuntime, tree: &TreeView) {
    let model = tree.model().unwrap();
    let (path, _) = tree.cursor();
    let iter = model.iter(&path.unwrap()).unwrap();
    let text = model.value(&iter, ID_COLUMN).get::<String>().unwrap();

    app_runtime.update_state_with(move |state| {
        let novel: Novel = state.get_by_id(text).unwrap();
        state.chapter_read_send(
            novel.settings.content_read.volumes,
            novel.settings.content_read.chapters - 1.0,
            novel.settings.content_read.side_stories,
            novel,
            true,
        );
    })
}

pub fn action_increase_read_count(app_runtime: &AppRuntime, tree: &TreeView) {
    let model = tree.model().unwrap();
    let (path, _) = tree.cursor();
    let iter = model.iter(&path.unwrap()).unwrap();
    let text = model.value(&iter, ID_COLUMN).get::<String>().unwrap();

    app_runtime.update_state_with(move |state| {
        let novel: Novel = state.get_by_id(text).unwrap();
        state.chapter_read_send(
            novel.settings.content_read.volumes,
            novel.settings.content_read.chapters + 1.0,
            novel.settings.content_read.side_stories,
            novel,
            true,
        );
    })
}

pub fn action_open_novel_dialog(
    app_runtime: &AppRuntime,
    tree: &TreeView,
    path: &TreePath,
    treeview_index: i32,
    edit: bool,
) {
    let model = tree.model().unwrap();
    let iter = model.iter(path).unwrap();

    let text = model.value(&iter, ID_COLUMN).get::<String>().unwrap();

    app_runtime.update_state_with(move |mut state| {
        if let Some(novel) = state.get_by_id(text) {
            // Use different values when opening the novel dialog from
            // the history treeview
            if treeview_index == 99 {
                state.ui.lists.active_list = ListStatus::from_i32(novel.settings.list_status.to_i32());
                state.ui.lists.active_iter = state.ui.lists.find_iter(&novel);
            } else {
                state.ui.lists.active_list = ListStatus::from_i32(treeview_index);
                state.ui.lists.active_iter = Some(iter);
            }

            state.ui.lists.active_novel = Some(novel.clone());

            if edit {
                state.ui.show_novel_dialog_edit()
            } else {
                state.ui.show_novel_dialog(&novel, &state.settings.read());
            }
        }
    });
}

/// Messily add columns
///
/// Code order decides the order of columns (for some reason)
pub fn add_columns(list_index: i32, tree: &gtk::TreeView, app_runtime: AppRuntime, settings: &Settings) {
    // Status column
    if settings.list.visible_columns[Column::Status as usize] {
        let renderer = gtk::CellRendererPixbuf::new();
        renderer.set_yalign(0.7);
        let column = cascade! {
            gtk::TreeViewColumn::new();
            ..pack_start(&renderer, true);
            ..set_max_width(22);
            ..add_attribute(&renderer, "pixbuf", Column::StatusIcon as i32);
            ..set_sort_column_id(Column::Status as i32);
            ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
        };
        tree.insert_column(&column, Column::Status as i32);
        tree.set_has_tooltip(true);
    }

    if settings.list.visible_columns[Column::OriginalLanguage as usize] {
        add_column(
            &app_runtime,
            tree,
            list_index,
            &fl!("column-ol"),
            Column::OriginalLanguage,
            *settings.list.column_width[list_index as usize]
                .get(&(Column::OriginalLanguage as i32))
                .unwrap_or(&40),
            0.5,
        );
    }

    if settings.list.visible_columns[Column::Title as usize] {
        add_column(
            &app_runtime,
            tree,
            list_index,
            &fl!("column-name"),
            Column::Title,
            *settings.list.column_width[list_index as usize]
                .get(&(Column::Title as i32))
                .unwrap_or(&310),
            0.0,
        );
    }

    // Column for chapters read
    if settings.list.visible_columns[Column::ChaptersRead as usize] {
        let renderer = gtk::CellRendererSpin::new();
        renderer.set_editable(true);
        renderer.connect_edited(
            glib::clone!(@strong app_runtime, @strong tree => move |_c, path, new_value| {
                let model = tree.model().unwrap();
                let iter = model.iter(&path).unwrap();
                let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
                let new_chapter_value = new_value.trim().replace(',', ".").parse::<f32>().unwrap();

                app_runtime.update_state_with(move |state| {
                    let novel = state.get_by_id(novel_id);
                    if let Some(novel) = novel {
                        state.chapter_read_send(
                            novel.settings.content_read.volumes,
                            new_chapter_value,
                            novel.settings.content_read.side_stories,
                            novel,
                            true
                        );
                    }
                });
            }),
        );
        renderer.set_adjustment(Some(&Adjustment::new(0.0, 0.0, 1_000_000.0, 1.0, 10.0, 0.0)));
        renderer.set_digits(1);
        renderer.set_climb_rate(1.0);

        renderer.set_padding(3, 1);
        // renderer.set_property_xalign(0.5);
        let tree_column = cascade! {
            gtk::TreeViewColumn::new();
            ..pack_start(&renderer, false);
            ..set_title(&fl!("column-chapters-read"));
            ..set_min_width(120);
            ..add_attribute(&renderer, "text", Column::ChaptersRead as i32);
            ..set_sort_column_id(Column::ChaptersRead as i32);
            ..set_resizable(true);
            ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
        };

        tree_column.connect_clicked(glib::clone!(@strong app_runtime => move |column| {
            let list_index = list_index;
            let sort_msg = SortingMessage {
                sorting: Sorting {
                    column_id: column.sort_column_id() as u32,
                    is_sort_indicator: column.is_sort_indicator(),
                    sort_order: match column.sort_order() {
                        gtk::SortType::Ascending => 1,
                        _ => 0,
                    }
                },
                list_index
            };

            app_runtime.update_state_with(move |state| {
                if state.list_populated {
                    state.list_sort_send(sort_msg);
                }
            });
        }));

        tree.insert_column(&tree_column, Column::ChaptersRead as i32);
    }

    // Column for side stories read
    if settings.list.visible_columns[Column::SideStoriesRead as usize] {
        let renderer = gtk::CellRendererSpin::new();
        renderer.set_editable(true);
        renderer.connect_edited(
            glib::clone!(@strong app_runtime, @strong tree => move |_c, path, new_value| {
                let model = tree.model().unwrap();
                let iter = model.iter(&path).unwrap();
                let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
                let new_side_stories_value = new_value.parse::<i32>().unwrap();

                app_runtime.update_state_with(move |state| {
                    let novel = state.get_by_id(novel_id);
                    if let Some(novel) = novel {
                        state.chapter_read_send(
                            novel.settings.content_read.volumes,
                            novel.settings.content_read.chapters,
                            new_side_stories_value,
                            novel,
                            true
                        );
                    }
                });
            }),
        );
        renderer.set_adjustment(Some(&Adjustment::new(0.0, 0.0, 1_000.0, 1.0, 10.0, 0.0)));
        renderer.set_padding(3, 1);
        // renderer.set_property_xalign(0.5);
        let tree_column = cascade! {
            gtk::TreeViewColumn::new();
            ..pack_start(&renderer, false);
            ..set_title(&fl!("column-side-stories-read"));
            ..set_min_width(120);
            ..add_attribute(&renderer, "text", Column::SideStoriesRead as i32);
            ..set_sort_column_id(Column::SideStoriesRead as i32);
            ..set_resizable(true);
            ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
        };

        tree_column.connect_clicked(glib::clone!(@strong app_runtime => move |column| {
            let list_index = list_index;
            let sort_msg = SortingMessage {
                sorting: Sorting {
                    column_id: column.sort_column_id() as u32,
                    is_sort_indicator: column.is_sort_indicator(),
                    sort_order: match column.sort_order() {
                        gtk::SortType::Ascending => 1,
                        _ => 0,
                    }
                },
                list_index
            };

            app_runtime.update_state_with(move |state| {
                if state.list_populated {
                    state.list_sort_send(sort_msg);
                }
            });
        }));

        tree.insert_column(&tree_column, Column::SideStoriesRead as i32);
    }

    // Column for volumes read
    if settings.list.visible_columns[Column::VolumesRead as usize] {
        let renderer = gtk::CellRendererSpin::new();
        renderer.set_editable(true);
        renderer.connect_edited(
            glib::clone!(@strong app_runtime, @strong tree => move |_c, path, new_value| {
            let model = tree.model().unwrap();
            let iter = model.iter(&path).unwrap();
            let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
            let new_volume_value = new_value.parse::<i32>().unwrap();

                app_runtime.update_state_with(move |state| {
                    let novel = state.get_by_id(novel_id);
                    if let Some(novel) = novel {
                        state.chapter_read_send(
                            new_volume_value,
                            novel.settings.content_read.chapters,
                            novel.settings.content_read.side_stories,
                            novel,
                            true
                        );
                    }
                });
            }),
        );

        renderer.set_adjustment(Some(&Adjustment::new(0.0, 0.0, 1_000.0, 1.0, 1.0, 0.0)));
        renderer.set_padding(3, 1);
        // renderer.set_property_xalign(0.5);
        let tree_column = cascade! {
            gtk::TreeViewColumn::new();
            ..pack_start(&renderer, false);
            ..set_title(&fl!("column-volumes-read"));
            ..set_min_width(120);
            ..add_attribute(&renderer, "text", Column::VolumesRead as i32);
            ..set_sort_column_id(Column::VolumesRead as i32);
            ..set_resizable(true);
            ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
        };

        tree_column.connect_clicked(glib::clone!(@strong app_runtime => move |column| {
            let list_index = list_index;
            let sort_msg = SortingMessage {
                sorting: Sorting {
                    column_id: column.sort_column_id() as u32,
                    is_sort_indicator: column.is_sort_indicator(),
                    sort_order: match column.sort_order() {
                        gtk::SortType::Ascending => 1,
                        _ => 0,
                    }
                },
                list_index
            };

            app_runtime.update_state_with(move |state| {
                if state.list_populated {
                    state.list_sort_send(sort_msg);
                }
            });
        }));

        tree.insert_column(&tree_column, Column::VolumesRead as i32);
    }

    if settings.list.visible_columns[Column::ChaptersAvailable as usize] {
        add_column(
            &app_runtime,
            tree,
            list_index,
            &fl!("column-availability"),
            Column::ChaptersAvailable,
            *settings.list.column_width[list_index as usize]
                .get(&(Column::ChaptersAvailable as i32))
                .unwrap_or(&0),
            0.0,
        );
    }
    if settings.list.visible_columns[Column::Score as usize] {
        add_column(
            &app_runtime,
            tree,
            list_index,
            &fl!("column-score"),
            Column::Score,
            *settings.list.column_width[list_index as usize]
                .get(&(Column::Score as i32))
                .unwrap_or(&0),
            0.0,
        );
    }
    if settings.list.visible_columns[Column::LastUpdate as usize] {
        add_column(
            &app_runtime,
            tree,
            list_index,
            &fl!("column-last-update"),
            Column::LastUpdate,
            *settings.list.column_width[list_index as usize]
                .get(&(Column::LastUpdate as i32))
                .unwrap_or(&160),
            0.0,
        );
    }
}

/// Adds one column that contains text
fn add_column(
    app_runtime: &AppRuntime,
    tree: &gtk::TreeView,
    list_index: i32,
    title: &str,
    column: Column,
    width: i32,
    x_align: f32,
) {
    let renderer = gtk::CellRendererText::new();
    renderer.set_padding(3, 1);
    renderer.set_property("ellipsize", gtk::pango::EllipsizeMode::End);
    gtk::prelude::CellRendererExt::set_alignment(&renderer, x_align, 0.5);

    let min_width = 40;
    // New column
    let tree_column = cascade! {
        gtk::TreeViewColumn::new();
        ..pack_start(&renderer, true);
        ..set_title(title);
        ..add_attribute(&renderer, "text", column as i32);
        ..set_sort_column_id(column as i32);
        ..set_min_width(min_width);
        ..set_resizable(true);
        ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
    };

    // Only update the fixed width if it is larger or same as the minimum value
    if width >= min_width {
        tree_column.set_fixed_width(width);
    }

    let col_id = column as i32;
    tree_column.connect_fixed_width_notify(glib::clone!(@strong app_runtime => move |column| {
        let list_index = list_index;
        let col_id = col_id;
        let col_width = column.width();
        app_runtime.update_state_with(move |state| {
            state.update_list_column_width(list_index, col_id, col_width);
        });
    }));

    tree_column.connect_clicked(glib::clone!(@strong app_runtime => move |column| {
        let list_index = list_index;
        let sort_msg = SortingMessage {
            sorting: Sorting {
                column_id: column.sort_column_id() as u32,
                is_sort_indicator: column.is_sort_indicator(),
                sort_order: match column.sort_order() {
                    gtk::SortType::Ascending => 1,
                    _ => 0,
                }
            },
            list_index
        };

        app_runtime.update_state_with(move |state| {
            if state.list_populated {
                state.list_sort_send(sort_msg);
            }
        });
    }));

    tree.insert_column(&tree_column, column as i32);
}

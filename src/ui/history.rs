use crate::app::history::NovelHistoryItem;
use crate::app::settings::Settings;
use crate::app::AppRuntime;
use crate::ui::novel_list::{action_open_novel_dialog, set_list_actions, ID_COLUMN};
use crate::utils::gtk::BuilderExtManualCustom;
use gdk::cairo::glib::SignalHandlerId;
use gio::prelude::*;
use glib::bitflags::_core::cmp::Ordering;
use glib::Type;
use gtk::{prelude::*, SortColumn};

#[derive(Debug, Clone)]
#[repr(i32)]
pub(crate) enum HistoryColumn {
    /// Novel title
    Title = 1,
    /// What happened
    Detail,
    /// When happened
    Time,
    /// For time sorting purposes
    Timestamp,
}

impl HistoryColumn {
    pub fn to_i32(&self) -> i32 {
        self.to_owned() as i32
    }

    pub fn to_u32(&self) -> u32 {
        self.to_owned() as u32
    }
}

pub struct HistoryList {
    pub treeview: gtk::TreeView,
    pub list: gtk::ListStore,
    pub mouse_handler: Option<SignalHandlerId>,
}

impl HistoryList {
    pub fn new(builder: &gtk::Builder) -> HistoryList {
        let list = gtk::ListStore::new(&[
            Type::STRING,
            Type::STRING,
            Type::STRING,
            Type::STRING,
            Type::I64,
        ]);

        let treeview = builder.get::<gtk::TreeView>("history_treeview");
        treeview.set_model(Some(&list));

        list.set_sort_column_id(
            gtk::SortColumn::Index(HistoryColumn::Time.to_u32()),
            gtk::SortType::Descending,
        );
        list.set_sort_func(
            SortColumn::Index(HistoryColumn::Time.to_u32()),
            list_sort_datetime,
        );

        HistoryList {
            treeview,
            list,
            mouse_handler: None,
        }
    }

    pub fn connect(&self, builder: &gtk::Builder, app_runtime: AppRuntime) {
        let treeview = builder.get::<gtk::TreeView>("history_treeview");
        treeview.connect_row_activated(
            glib::clone!(@strong app_runtime => move |tree, path, _col| {
                action_open_novel_dialog(&app_runtime, tree, path, 99, false);
            }),
        );
    }

    pub fn connect_mouse_actions(
        &mut self,
        builder: &gtk::Builder,
        app_runtime: AppRuntime,
        settings: &Settings,
    ) {
        let treeview = builder.get::<gtk::TreeView>("history_treeview");
        // If the handler exists then disconnect it and get rid of it
        if let Some(handler) = self.mouse_handler.take() {
            treeview.disconnect(handler);
        }
        // New handler for mouse actions
        let handler = treeview.connect_button_press_event(
            glib::clone!(@strong app_runtime, @strong settings => move |tree, event| {
                set_list_actions(&app_runtime, event, &settings, tree, 99);

                gtk::Inhibit(false)
            }),
        );
        // Save the handler for later disconnecting
        self.mouse_handler = Some(handler);
    }

    pub fn add_columns(&self, builder: &gtk::Builder) {
        let treeview = builder.get::<gtk::TreeView>("history_treeview");
        add_history_columns(&self.list, &treeview);
    }

    pub fn populate_columns(&self, data: &[NovelHistoryItem]) {
        for (_d_index, history) in data.iter().enumerate() {
            let values: [(u32, &dyn ToValue); 5] = [
                (ID_COLUMN as u32, &history.novel_id),
                (HistoryColumn::Title.to_u32(), &history.novel_name),
                (HistoryColumn::Detail.to_u32(), &history.detail_string()),
                (HistoryColumn::Time.to_u32(), &history.time_string()),
                (HistoryColumn::Timestamp.to_u32(), &history.time),
            ];

            self.list.set(&self.list.append(), &values);
        }
    }

    pub fn list_insert(&mut self, history: &NovelHistoryItem) {
        let values: [(u32, &dyn ToValue); 5] = [
            (ID_COLUMN as u32, &history.novel_id),
            (HistoryColumn::Title.to_u32(), &history.novel_name),
            (HistoryColumn::Detail.to_u32(), &history.detail_string()),
            (HistoryColumn::Time.to_u32(), &history.time_string()),
            (HistoryColumn::Timestamp.to_u32(), &history.time),
        ];

        self.list.insert_with_values(Some(0), &values);

        self.scroll_to_top();
    }

    /// Does what is says, sometimes.
    pub fn scroll_to_top(&self) {
        self.treeview.scroll_to_point(0, 0);
    }
}

pub fn add_history_columns(_model: &gtk::ListStore, treeview: &gtk::TreeView) {
    // Column for name
    {
        let renderer = gtk::CellRendererText::new();
        renderer.set_padding(3, 1);
        let column = cascade! {
            gtk::TreeViewColumn::new();
            ..pack_start(&renderer, true);
            ..set_title(&fl!("column-name"));
            ..set_min_width(340);
            ..add_attribute(&renderer, "text", HistoryColumn::Title as i32);
            ..set_sort_column_id(HistoryColumn::Title as i32);
            ..set_resizable(true);
            ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
        };
        treeview.append_column(&column);
    }

    // Column for detail
    {
        let renderer = gtk::CellRendererText::new();
        renderer.set_padding(3, 1);
        let column = cascade! {
            gtk::TreeViewColumn::new();
            ..pack_start(&renderer, true);
            ..set_title(&fl!("column-detail"));
            ..set_min_width(140);
            ..add_attribute(&renderer, "text", HistoryColumn::Detail as i32);
            ..set_sort_column_id(HistoryColumn::Detail as i32);
            ..set_resizable(true);
            ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
        };
        treeview.append_column(&column);
    }

    // Column for last update
    {
        let renderer = gtk::CellRendererText::new();
        renderer.set_padding(3, 1);
        let column = cascade! {
            gtk::TreeViewColumn::new();
            ..pack_start(&renderer, true);
            ..set_title(&fl!("column-last-update"));
            ..add_attribute(&renderer, "text", HistoryColumn::Time as i32);
            ..set_sort_order(gtk::SortType::Descending);
            ..set_sort_column_id(HistoryColumn::Time as i32);
            ..set_resizable(true);
            ..set_sizing(gtk::TreeViewColumnSizing::Fixed);
        };
        treeview.append_column(&column);
    }
}

/// Sort function for history list which sorts the `HistoryColumn::Time` column
/// based on hidden `HistoryColumn::Timestamp` data.
fn list_sort_datetime(
    model: &gtk::TreeModel,
    a_iter: &gtk::TreeIter,
    b_iter: &gtk::TreeIter,
) -> Ordering {
    let date_a = model
        .value(a_iter, HistoryColumn::Timestamp.to_i32())
        .get::<i64>()
        .unwrap();
    let date_b = model
        .value(b_iter, HistoryColumn::Timestamp.to_i32())
        .get::<i64>()
        .unwrap();

    date_a.cmp(&date_b)
}

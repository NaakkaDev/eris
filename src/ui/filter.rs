use crate::app::novel::Novel;
use crate::app::settings::Settings;
use crate::app::AppRuntime;
use crate::ui::novel_list::{
    action_open_novel_dialog, add_columns, list_sort_datetime, set_list_actions, Column,
    COLUMN_COUNT, COLUMN_TYPES, ID_COLUMN,
};
use crate::utils::gtk::BuilderExtManualCustom;
use gdk::cairo::glib::SignalHandlerId;
use gio::prelude::*;
use gtk::{prelude::*, SortColumn};

pub struct FilterList {
    pub treeview: gtk::TreeView,
    pub list: gtk::ListStore,
    pub model: gtk::TreeModelFilter,
    pub entry: gtk::SearchEntry,

    pub selected_page: i32,
    pub mouse_handler: Option<SignalHandlerId>,
    pub tooltip_handler: Option<SignalHandlerId>,
}

impl FilterList {
    pub fn new(builder: &gtk::Builder) -> FilterList {
        let treeview = builder.get::<gtk::TreeView>("TreeView6");
        let list = gtk::ListStore::new(&COLUMN_TYPES);
        let entry = builder.get::<gtk::SearchEntry>("filter_entry");
        let model = gtk::TreeModelFilter::new(&list, None);

        // Set the filter model to the filter treeview
        treeview.set_search_entry(Some(&entry));
        treeview.set_model(Some(&model));

        let last_update_column_id = Column::LastUpdate as u32;
        list.set_sort_func(SortColumn::Index(last_update_column_id), list_sort_datetime);

        FilterList {
            treeview,
            list,
            model,
            entry,

            selected_page: 0,
            mouse_handler: None,
            tooltip_handler: None,
        }
    }

    pub fn connect(&self, app_runtime: AppRuntime, list_notebook: &gtk::Notebook) {
        self.treeview.connect_row_activated(
            glib::clone!(@strong app_runtime => move |tree, path, _col| {
                action_open_novel_dialog(&app_runtime, tree, path, 5, false);
            }),
        );

        // Set the filter model visibility function
        let filter_entry = self.entry.clone();
        self.model.set_visible_func(move |model, iter| {
            let novel_name = model.value(iter, Column::Title as i32).get::<String>();
            if let Ok(name) = novel_name {
                let entry_query = filter_entry.text().to_string();
                if name.to_lowercase().contains(&entry_query.to_lowercase()) {
                    return true;
                }
            }

            false
        });

        // When filter search entry changes and has characters
        // then show the filter tab, if there are no characters
        // then go back to the previously selected tab
        self.entry.connect_changed(glib::clone!(@strong app_runtime, @strong list_notebook as notebook, @strong self.model as filter_model, @strong self.selected_page as selected_page => move |entry| {
            filter_model.refilter();
            let query = entry.text().to_string();

            if notebook.page() == 5 && query.is_empty() {
                app_runtime.update_state_with(|state| {
                    state.return_to_selected_page();
                });
            }
            else if notebook.page() != 5 && !query.is_empty() {
                app_runtime.update_state_with(|state| {
                    state.ui.show_novel_list_no_focus();
                    state.ui.list_notebook.set_page(5);
                });
            }
        }));

        self.entry.connect_key_press_event(|entry, e| {
            if let Some(keycode) = e.keycode() {
                // ESC
                if keycode == 27 {
                    // Clear the entry when Esc is pressed
                    entry.set_text("");
                }
            }
            gtk::Inhibit(false)
        });
    }

    pub fn connect_mouse_actions(&mut self, app_runtime: AppRuntime, settings: &Settings) {
        let treeview = &self.treeview;
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

    pub fn connect_tooltips(&mut self, settings: &Settings) {
        // Disconnect any stale-to-be tooltip events
        if let Some(handler) = self.tooltip_handler.take() {
            self.treeview.disconnect(handler);
        }

        // Only the status column has tooltips so if they are not visible then do nothing else
        if !settings.list.visible_columns[Column::Status as usize] {
            return;
        }

        let handler = self
            .treeview
            .connect_query_tooltip(|tree, _x, y, _keyboard, tooltip| {
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
                    let novel_status = model
                        .value(&iter, Column::Status as i32)
                        .get::<String>()
                        .unwrap();

                    tooltip.set_text(Some(&novel_status));
                    tree.set_tooltip_cell(tooltip, None, Some(&column), Some(&cell));

                    return true;
                }

                false
            });

        self.tooltip_handler = Some(handler);
    }

    pub fn add_columns(&mut self, app_runtime: AppRuntime, settings: &Settings) {
        // Remove any existing columns
        for col in self.treeview.columns() {
            self.treeview.remove_column(&col);
        }

        add_columns(5, &self.treeview, app_runtime, settings);

        self.connect_tooltips(settings);
    }

    pub fn populate_columns(&self, novel_data: &[Novel]) {
        for (_, novel) in novel_data.iter().enumerate() {
            let values: [(u32, &dyn ToValue); COLUMN_COUNT] = [
                (ID_COLUMN as u32, &novel.id),
                (Column::Status as u32, &novel.status.to_string()),
                (Column::StatusIcon as u32, &novel.status_pix()),
                (Column::OriginalLanguage as u32, &novel.orig_lang()),
                (Column::Title as u32, &novel.title()),
                (
                    Column::VolumesRead as u32,
                    &novel.settings.content_read.volumes,
                ),
                (Column::ChaptersRead as u32, &novel.chapters_read_str()),
                (
                    Column::SideStoriesRead as u32,
                    &novel.settings.content_read.side_stories,
                ),
                (Column::ChaptersAvailable as u32, &novel.content()),
                (Column::Score as u32, &novel.settings.score),
                (
                    Column::LastUpdate as u32,
                    &novel.settings.last_updated_string(),
                ),
            ];

            self.list.set(&self.list.append(), &values);
        }
    }

    pub fn list_insert(&mut self, novel: &Novel) {
        let values: [(u32, &dyn ToValue); COLUMN_COUNT] = [
            (ID_COLUMN as u32, &novel.id),
            (Column::Status as u32, &novel.status.to_string()),
            (Column::StatusIcon as u32, &novel.status_pix()),
            (Column::OriginalLanguage as u32, &novel.orig_lang()),
            (Column::Title as u32, &novel.title()),
            (
                Column::VolumesRead as u32,
                &novel.settings.content_read.volumes,
            ),
            (Column::ChaptersRead as u32, &novel.chapters_read_str()),
            (
                Column::SideStoriesRead as u32,
                &novel.settings.content_read.side_stories,
            ),
            (Column::ChaptersAvailable as u32, &novel.content()),
            (Column::Score as u32, &novel.settings.score),
            (
                Column::LastUpdate as u32,
                &novel.settings.last_updated_string(),
            ),
        ];

        self.list.insert_with_values(Some(0), &values);
    }

    pub fn list_remove(&mut self, novel: &Novel) {
        let tree_iter = self.find_iter(novel);
        if let Some(iter) = &tree_iter {
            self.list.remove(iter);
        } else {
            error!(
                "Cannot remove novel {} from filter list, iter not found.",
                novel.id
            );
        }
    }

    pub fn list_update(&mut self, novel: &Novel) {
        let tree_iter = self.find_iter(novel);
        if let Some(iter) = &tree_iter {
            self.list.set_value(
                iter,
                Column::Status as u32,
                &novel.status.to_str().to_value(),
            );
            self.list.set_value(
                iter,
                Column::StatusIcon as u32,
                &novel.status_pix().to_value(),
            );
            self.list
                .set_value(iter, Column::Title as u32, &novel.title().to_value());
            self.list.set_value(
                iter,
                Column::ChaptersRead as u32,
                &novel.chapters_read_str().to_value(),
            );
            self.list.set_value(
                iter,
                Column::VolumesRead as u32,
                &novel.settings.content_read.volumes.to_value(),
            );
            self.list.set_value(
                iter,
                Column::ChaptersAvailable as u32,
                &novel.content().to_value(),
            );
            self.list.set_value(
                iter,
                Column::SideStoriesRead as u32,
                &novel.settings.content_read.side_stories.to_value(),
            );
            self.list
                .set_value(iter, Column::Score as u32, &novel.settings.score.to_value());
            self.list.set_value(
                iter,
                Column::LastUpdate as u32,
                &novel.settings.last_updated_string().to_value(),
            );
        }
    }

    pub fn scroll_to_top(&self) {
        self.treeview.scroll_to_point(0, 2);
    }

    pub fn find_iter(&self, novel: &Novel) -> Option<gtk::TreeIter> {
        let model = &self.model;
        for i in 0..(model.iter_n_children(None)) {
            // Check if this is the correct iter
            let iter = model.iter_from_string(&i.to_string());
            if let Some(iter) = iter {
                // If the novel id in the row matches with the novel id being read then the correct one was found.
                let novel_id = model.value(&iter, ID_COLUMN).get::<String>().unwrap();
                if novel_id == novel.id {
                    let path = model.path(&iter).unwrap();
                    return self.list.iter(&path);
                }
            }
        }

        None
    }
}

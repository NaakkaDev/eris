use crate::app::history::NovelHistoryItem;
use crate::appop::AppOp;

impl AppOp {
    /// Message recepient. Adds a new `NovelHistoryItem` to UI, list and file.
    pub fn new_history_item(&mut self, history_item: NovelHistoryItem) {
        debug!("appop::history::new_history_item");

        // Add to UI
        self.ui.history.list_insert(&history_item);
        // Add to list and save to file
        self.history.write().items.push(history_item);
        self.history
            .write()
            .write_to_file()
            .expect("Cannot write history to file");
    }
}

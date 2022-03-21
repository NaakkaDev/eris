use crate::app::history::NovelHistoryItem;
use crate::appop::AppOp;

impl AppOp {
    /// Message recepient. Adds a new `NovelHistoryItem` to UI, list and file.
    pub fn new_history_item(&mut self, history_item: NovelHistoryItem) {
        debug!("appop::history::new_history_item");

        // Do nothing if the last entry in history is identical, time excluded
        if let Some(last) = self.history.read().items.last() {
            if last.novel_id == history_item.novel_id
                && last.action == history_item.action
                && last.named_chapter == history_item.named_chapter
                && last.content == history_item.content
            {
                return;
            }
        }

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

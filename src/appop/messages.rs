use crate::app::history::NovelHistoryItem;
use crate::app::novel::{ChapterRead, Novel};
use crate::app::settings::Sorting;
use crate::appop::AppOp;

impl AppOp {
    /// Create the `chapter_read` message.
    pub fn chapter_read_message(&self) -> glib::Sender<ChapterRead> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        rx.attach(None, glib::clone!(@strong self.app_runtime as app_runtime => @default-return glib::Continue(false), move |data: ChapterRead| {
            app_runtime.update_state_with(move |state| {
                state.chapter_read(data.clone());
                state.previous_chapter_read = Some(data);
            });

            glib::Continue(true)
        }));

        tx
    }

    /// Send the `chapter_read` message.
    pub fn chapter_read_send(
        &self,
        volume: i32,
        chapter: f32,
        side: i32,
        novel: Novel,
        exact_num: bool,
    ) {
        self.chapter_read_sender
            .as_ref()
            .unwrap()
            .send(ChapterRead {
                volume,
                chapter,
                side,
                exact_num,
                novel,
            })
            .expect("Cannot send ChapterRead message");
    }

    pub fn history_message(&self) -> glib::Sender<NovelHistoryItem> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        rx.attach(None, glib::clone!(@strong self.app_runtime as app_runtime => @default-return glib::Continue(false), move |data: NovelHistoryItem| {
            app_runtime.update_state_with(move |state| {
                state.new_history_item(data);
            });

            glib::Continue(true)
        }));

        tx
    }

    /// Used for sending `NovelHistoryItem` message every time a new history
    /// entry needs to be added.
    pub fn history_send(&self, history_item: NovelHistoryItem) {
        self.history_sender
            .as_ref()
            .unwrap()
            .send(history_item)
            .expect("Cannot send ChapterRead message");
    }

    pub fn list_sort_message(&self) -> glib::Sender<SortingMessage> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        rx.attach(None, glib::clone!(@strong self.app_runtime as app_runtime => @default-return glib::Continue(false), move |data| {
            app_runtime.update_state_with(move |state| {
                state.update_list_sort_order(data);
            });

            glib::Continue(true)
        }));

        tx
    }

    pub fn list_sort_send(&self, list_sort: SortingMessage) {
        self.list_sort_sender
            .as_ref()
            .unwrap()
            .send(list_sort)
            .expect("Cannot send ChapterRead message");
    }
}

#[derive(Debug)]
pub struct SortingMessage {
    pub sorting: Sorting,
    pub list_index: i32,
}

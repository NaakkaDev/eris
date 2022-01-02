use crate::utils::Resources;
use glib::{IsA, Object};
use gtk::prelude::{BuilderExt, BuilderExtManual, ButtonExt, GtkMenuItemExt, LabelExt};
use gtk::Builder;

/// Helper trait & methods to make it a bit easier to handle objects from ui file.
///
/// Before:
/// ```
/// let button = builder
///     .object::<gtk::Button>("button")
///     .expect("Cannot find button in ui file.");
/// ```
/// After:
/// ```
/// let button = builder.get::<gtk::Button>("button");
/// ```
pub trait BuilderExtManualCustom {
    fn get<T: IsA<Object>>(&self, name: &str) -> T;

    fn from_resources(file: &str) -> Builder;

    fn add_from_resources(&self, file: &str);

    fn label_i18n(&self, name: &str, label_text: &str);

    fn button_i18n(&self, name: &str, button_text: &str);

    fn menu_item_i18n(&self, name: &str, label_text: &str);

    fn menu_checkitem_i18n(&self, name: &str, label_text: &str);

    fn checkbutton_i18n(&self, name: &str, label_text: &str);
}

impl<O: IsA<Builder>> BuilderExtManualCustom for O {
    fn get<T: IsA<Object>>(&self, name: &str) -> T {
        self.object::<T>(name)
            .unwrap_or_else(|| panic!("Cannot find {} in ui file.", name))
    }

    fn from_resources(file: &str) -> Builder {
        let resource =
            String::from_utf8(Resources::get(file).unwrap().data.as_ref().to_vec()).unwrap();

        Builder::from_string(&resource)
    }

    fn add_from_resources(&self, file: &str) {
        let resource =
            String::from_utf8(Resources::get(file).unwrap().data.as_ref().to_vec()).unwrap();

        self.add_from_string(&resource)
            .unwrap_or_else(|_| panic!("Cannot add {} from string", file));
    }

    fn label_i18n(&self, name: &str, label_text: &str) {
        self.get::<gtk::Label>(name).set_text(label_text);
    }

    fn button_i18n(&self, name: &str, button_text: &str) {
        self.get::<gtk::Button>(name).set_label(button_text);
    }

    fn menu_item_i18n(&self, name: &str, label_text: &str) {
        self.get::<gtk::MenuItem>(name).set_label(label_text);
    }

    fn menu_checkitem_i18n(&self, name: &str, label_text: &str) {
        self.get::<gtk::CheckMenuItem>(name).set_label(label_text);
    }

    fn checkbutton_i18n(&self, name: &str, label_text: &str) {
        self.get::<gtk::CheckButton>(name).set_label(label_text);
    }
}

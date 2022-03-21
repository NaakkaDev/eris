use crate::app::novel::Novel;
use crate::ui::UI;

use crate::app::AppRuntime;
use crate::appop::novel_recognition::NovelRecognitionData;
use crate::data_dir;
use crate::utils::gtk::BuilderExtManualCustom;
use gdk::pango::WrapMode;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{Align, IconSize, Orientation};

impl UI {
    pub fn show_reading_not(&self) {
        // Change the reading notebook page
        // but do not change the main notebook page
        self.reading_notebook.set_current_page(Some(0));
    }

    pub fn show_reading_now_reading(&self) {
        let reading_now_listboxrow = self
            .builder
            .get::<gtk::ListBoxRow>("reading_now_listboxrow");
        reading_now_listboxrow.activate();

        // Show the reading now page
        self.reading_notebook.set_current_page(Some(1));
        self.main_notebook.set_current_page(Some(0));
    }

    /// Update currently reading UI elements novel name and chapter number.
    pub fn update_currently_reading(&self, novel: &Novel) {
        let previously_read_title_label = self
            .builder
            .get::<gtk::Label>("previously_read_title_label");
        let previously_read_chapter_label = self
            .builder
            .get::<gtk::Label>("previously_read_chapter_label");
        let button = self.builder.get::<gtk::Button>("btn_continue_reading");

        previously_read_title_label.set_text(&novel.title);

        let mut prev_read = String::new();
        if novel.settings.content_read.volumes > 0 {
            prev_read.push_str(&format!(
                "{} {}\n",
                &fl!("volume"),
                &novel.settings.content_read.volumes
            ));
        }
        if novel.settings.content_read.chapters > 0.0 {
            prev_read.push_str(&format!(
                "{} {}\n",
                &fl!("chapter"),
                &novel.settings.content_read.chapters
            ));
        }
        if novel.settings.content_read.side_stories > 0 {
            prev_read.push_str(&format!(
                "{} {}\n",
                &fl!("side-story"),
                &novel.settings.content_read.side_stories
            ));
        }
        previously_read_chapter_label.set_text(&prev_read);
        // Button is disabled by default so make it clickable
        button.set_sensitive(true);
    }

    pub fn update_reading_now_volume(&self, volume_num: i32, volume_avail: i32) {
        let mut reading_now_vol = String::new();
        reading_now_vol.push_str(&volume_num.to_string());
        if volume_avail > 0 {
            reading_now_vol.push_str(" / ");
            reading_now_vol.push_str(&volume_avail.to_string());
        }

        let reading_volume_text = self.builder.get::<gtk::Label>("reading_volume_label");
        let reading_volume_number = self.builder.get::<gtk::Label>("reading_volume_number");

        reading_volume_text.set_visible(volume_num > 0);
        reading_volume_number.set_visible(volume_num > 0);

        reading_volume_number.set_label(&reading_now_vol);
    }

    pub fn update_reading_now_chapter(&self, chapter_num: f32, chapter_avail: f32) {
        let mut reading_now_ch = String::new();
        reading_now_ch.push_str(&chapter_num.to_string());
        if chapter_avail > 0.0 {
            reading_now_ch.push_str(" / ");
            reading_now_ch.push_str(&chapter_avail.to_string());
        }

        let reading_chapter_number = self.builder.get::<gtk::Label>("reading_chapter_number");

        reading_chapter_number.set_label(&reading_now_ch);
    }

    pub fn update_reading_now_side_stories(&self, side_story_num: i32, side_story_avail: i32) {
        let mut reading_now_ss = String::new();
        reading_now_ss.push_str(&side_story_num.to_string());
        if side_story_avail > 0 {
            reading_now_ss.push_str(" / ");
            reading_now_ss.push_str(&side_story_avail.to_string());
        }

        let reading_side_story_text = self.builder.get::<gtk::Label>("reading_side_story_label");
        let reading_side_story_number = self.builder.get::<gtk::Label>("reading_side_story_number");

        reading_side_story_text.set_visible(side_story_num > 0);
        reading_side_story_number.set_visible(side_story_num > 0);

        reading_side_story_number.set_label(&reading_now_ss);
    }

    /// Update reading now view which contains minimal novel information and
    /// current chapter/volume being read.
    pub fn update_reading_now(
        &mut self,
        novel: &Option<Novel>,
        novel_name: &str,
        data: &NovelRecognitionData,
        source: &str,
    ) {
        let image = self.builder.get::<gtk::Image>("reading_novel_image");
        let reading_novel_title = self.builder.get::<gtk::Label>("reading_novel_title_label");
        let reading_novel_title_source =
            self.builder.get::<gtk::Label>("reading_novel_source_label");
        let novel_info_box = self.builder.get::<gtk::Box>("novel_info_box");
        let reading_novel_alt_title_text = self
            .builder
            .get::<gtk::TextView>("reading_novel_alt_title_text");
        let reading_novel_detail_author_value = self
            .builder
            .get::<gtk::Label>("reading_novel_detail_author_value_label");
        let reading_novel_detail_artist_value = self
            .builder
            .get::<gtk::Label>("reading_novel_detail_artist_value_label");
        let reading_novel_detail_genre_value = self
            .builder
            .get::<gtk::Label>("reading_novel_detail_genre_value_label");
        let reading_novel_description_text = self
            .builder
            .get::<gtk::TextView>("reading_novel_description_text");
        let reading_novel_slug = self.builder.get::<gtk::Label>("reading_novel_slug");
        let novel_not_found_box = self.builder.get::<gtk::Box>("novel_not_found_box");
        let reading_type = self.builder.get::<gtk::Label>("reading_type");
        let reading_grid = self.builder.get::<gtk::Grid>("reading_grid");
        let alt_titles_box = self.builder.get::<gtk::Box>("alt_titles_box");
        let reading_artist_box = self.builder.get::<gtk::Box>("reading_artist_box");

        let mut vol_avail = 0;
        let mut ch_avail = 0.0;
        let mut ss_avail = 0;

        // Set the reading grid element visible if reading
        // and the program has found potential sensible
        // chapter numbers, otherwise set it to false as it is not needed
        if data.chapter == 0.0 && data.side_story == 0 && data.volume == 0 {
            reading_grid.set_visible(false);
        } else {
            reading_grid.set_visible(data.reading);
        }

        image.set_from_icon_name(Some("gtk-missing-image"), IconSize::Dialog);

        if let Some(chapter_title) = data.chapter_title.clone() {
            reading_novel_title_source.set_label(&format!(
                "{} \"{}\" on {}",
                fl!("reading"),
                chapter_title,
                source
            ));
        } else {
            reading_novel_title_source.set_label(source);
        }

        if let Some(novel) = novel {
            novel_not_found_box.set_visible(false);
            novel_info_box.set_visible(true);

            vol_avail = novel.content.volumes;
            ch_avail = novel.content.chapters;
            ss_avail = novel.content.side_stories;

            if let Some(image_path) = novel.image.first() {
                let full_path = &data_dir(image_path);
                if full_path.exists() {
                    let pb = Pixbuf::from_file_at_scale(full_path, 150, 215, false)
                        .expect("Cannot get Pixbuf");

                    image.set_from_pixbuf(Some(&pb));
                }
            }

            alt_titles_box.set_visible(novel.alternative_titles.is_some());
            if let Some(alt_titles) = &novel.alternative_titles {
                reading_novel_alt_title_text
                    .buffer()
                    .expect("Cannot get buffer")
                    .set_text(&alt_titles.join("\n"));
            }

            let novel_type = format!("{} - {}", novel.novel_type.to_string(), novel.status());

            // Hide artist row if empty
            reading_artist_box.set_visible(!novel.artist.is_empty());

            reading_novel_title.set_label(&novel.title);
            reading_novel_detail_author_value.set_text(&novel.authors());
            reading_novel_detail_artist_value.set_text(&novel.artists());
            reading_novel_detail_genre_value.set_text(&novel.genres());
            reading_type.set_text(&novel_type);

            if let Some(description) = &novel.description {
                reading_novel_description_text
                    .buffer()
                    .expect("Could not get buffer")
                    .set_text(description);
            } else {
                reading_novel_alt_title_text
                    .buffer()
                    .expect("Cannot get buffer")
                    .set_text(&fl!("no-description"));
            }

            // Disconnect link handler if one exists
            if let Some(link_handler) = self.link_handler.take() {
                reading_novel_slug.disconnect(link_handler);
            }

            if let Some(slug) = &novel.slug {
                reading_novel_slug.set_markup(&format!("<a href=\"{}\">{}</a>", slug, slug));

                let novel_clone = novel.clone();
                let handler = reading_novel_slug.connect_activate_link(move |_, _| {
                    novel_clone.open_slug();

                    gtk::Inhibit(true)
                });
                self.link_handler = Some(handler);
            } else {
                reading_novel_slug.set_markup("");
            }
        } else {
            novel_not_found_box.set_visible(true);
            novel_info_box.set_visible(false);

            reading_novel_title.set_label(novel_name);
            reading_type.set_text("");

            if self.reading_notebook.current_page() != Some(1) {
                self.reading_notebook.set_current_page(Some(1));
            }
        }

        self.update_reading_now_volume(data.volume, vol_avail);
        self.update_reading_now_chapter(data.chapter, ch_avail);
        self.update_reading_now_side_stories(data.side_story, ss_avail);
    }

    /// Show a `gtk::box` containing potential novel suggestions
    /// if the supplied `novels` is not empty.
    pub fn show_potential_novels(
        &self,
        keyword: String,
        novels: Vec<Novel>,
        app_runtime: AppRuntime,
    ) {
        let potential_box = self.builder.get::<gtk::Box>("potential_box");
        let potential_novels_box = self.builder.get::<gtk::Box>("potential_novels_box");

        // Remove any existing labels
        for child in potential_novels_box.children() {
            potential_novels_box.remove(&child);
        }

        if novels.is_empty() {
            potential_box.set_visible(false);
            return;
        } else {
            potential_box.set_visible(true);
        }

        for novel in novels {
            let spinner = gtk::Spinner::new();

            let label_novel_text = format!("-> {}", novel.title);
            let label_novel = cascade! {
                gtk::Label::new(Some(&label_novel_text));
                ..set_wrap(true);
                ..set_wrap_mode(WrapMode::Word);
                ..set_selectable(true);
                ..set_focus_on_click(true);
                ..set_xalign(0.0);
            };

            let label_link_text = fl!("add-to-keywords", k = keyword.clone());
            let link_btn = cascade! {
                gtk::LinkButton::new("");
                ..set_label(&label_link_text);
                ..set_halign(Align::Start);
                ..set_has_tooltip(false);
                ..set_widget_name("thin-link-btn");
                ..connect_activate_link(glib::clone!(@strong app_runtime, @strong keyword, @strong spinner => move |btn| {
                    // Show the spinner when link button is clicked
                    spinner.set_active(true);
                    btn.set_visible(false);

                    let novel_id = novel.id.clone();
                    let keyword = keyword.clone();
                    app_runtime.update_state_with(|state| {
                        if let Some(novel) = state.get_by_id(novel_id) {
                            state.update_novel_reading_keyword(novel.clone(), keyword);
                            let novel_title = novel.clone().title;
                            state.ui.update_reading_now(&Some(novel), &novel_title, &NovelRecognitionData::default(), "");
                        }
                    });
                    gtk::Inhibit(true)
                }));
            };

            // Box for link button and spinner
            let spinner_box = cascade! {
                gtk::Box::new(Orientation::Horizontal, 20);
                ..add(&link_btn);
                ..add(&spinner);
            };

            let vbox = cascade! {
                gtk::Box::new(Orientation::Vertical, 0);
                ..set_margin_bottom(10);
                ..add(&label_novel);
                ..add(&spinner_box);
            };

            potential_novels_box.add(&vbox);
        }

        potential_novels_box.show_all();
    }
}

use crate::app::novel::{Novel, NovelContentAmount, NovelSettings, NovelType};
use crate::appop::parsers::{cover_image_file, ParseNovel};
use chrono::{Datelike, Local, NaiveDateTime};
use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use std::str::FromStr;

pub struct ScribbleHub {
    pub document: Document,
}

impl ScribbleHub {
    pub fn new(document: Document) -> Self {
        Self { document }
    }
}

impl ParseNovel for ScribbleHub {
    fn parse_novel(&self, slug: &str) -> Option<Novel> {
        let novel_title = self.parse_title();
        let novel_id = self.generate_id(&novel_title);
        let image = self.parse_image(&novel_id);

        let content = NovelContentAmount {
            chapters: self.parse_chapters(&[]) as f32,
            side_stories: self.parse_side_stories(&[]),
            volumes: self.parse_volumes(&[]),
        };

        let status_strings = self
            .document
            .select(Class("copyright"))
            .into_iter()
            .map(|node| node.text())
            .collect::<Vec<String>>();

        let status_strs = status_strings.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        let novel = Novel {
            id: novel_id,
            title: novel_title,
            image,
            alternative_titles: None,
            description: Some(self.parse_description()),
            author: self.parse_author(),
            artist: self.parse_artist(),
            genre: self.parse_genre(),
            tags: self.parse_tags(),
            novel_type: self.parse_type(),
            original_language: self.parse_original_language(),
            translated: self.parse_translated(),
            content,
            status: self.parse_status(&status_strs),
            year: self.parse_year(),
            original_publisher: self.parse_original_publisher(),
            english_publisher: self.parse_english_publisher(),
            source: Some(self.generate_source(slug)),
            slug: Some(slug.to_string()),
            last_scrape: Local::now().timestamp(),
            settings: NovelSettings::default(),
        };

        Some(novel)
    }

    fn parse_title(&self) -> String {
        self.document.select(Class("fic_title")).next().unwrap().text()
    }

    fn parse_image(&self, novel_id: &str) -> Vec<String> {
        let image_url = self
            .document
            .select(Class("fic_image").descendant(Name("img")))
            .next()
            .unwrap()
            .attr("src")
            .unwrap();

        // Get the cover image if there is one available
        if !image_url.contains("noimage") {
            let image_name = sanitize_filename::sanitize(&novel_id);
            let image_path = cover_image_file(image_url, image_name.as_str());
            return vec![image_path];
        }

        vec![]
    }

    fn parse_description(&self) -> String {
        let novel_description_vec = self
            .document
            .select(Class("wi_fic_desc"))
            .next()
            .unwrap()
            .text()
            .trim()
            .replace("\n", "\n\n")
            .split(' ')
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        novel_description_vec.join(" ")
    }

    fn parse_author(&self) -> Vec<String> {
        let author = self.document.select(Class("auth_name_fic")).next().unwrap().text();

        vec![author]
    }

    fn parse_genre(&self) -> Vec<String> {
        self.document
            .select(Class("wi_fic_genre").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text())
            .collect()
    }

    fn parse_tags(&self) -> Vec<String> {
        self.document
            .select(Class("wi_fic_showtags").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text())
            .collect()
    }

    fn parse_type(&self) -> NovelType {
        NovelType::from_str("Web Novel").unwrap()
    }

    fn parse_original_language(&self) -> String {
        "English".to_string()
    }

    fn parse_chapters(&self, _strings: &[&str]) -> i32 {
        self.document
            .select(Class("cnt_toc"))
            .next()
            .unwrap()
            .text()
            .parse()
            .unwrap_or(0)
    }

    fn parse_year(&self) -> i32 {
        let first_chapter_time = self
            .document
            .select(Class("toc_ol").descendant(Class("fic_date_pub")))
            .last()
            .unwrap()
            .attr("title")
            .unwrap();

        let dt = NaiveDateTime::parse_from_str(first_chapter_time, "%b %d, %Y %l:%M %p");
        dt.unwrap().year()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::appop::parsers::NovelParser;
    use std::fs;
    use std::path::Path;

    /// Download any novel page as htm from www.novelupdates.com
    /// and rename it to be `scribblehub.htm` and put it in the project root directory.
    #[test]
    fn test_parser() {
        let test_file = "scribblehub.htm";

        if !Path::new(test_file).exists() {
            return;
        }

        let file = fs::read_to_string(test_file).expect("Unable to read file");
        let document = Document::from(file.as_str());

        let novel = NovelParser::ScribbleHub.parse(document, "https://www.scribblehub.com");

        println!("{:?}", novel);

        assert!(novel.is_some());
    }
}

use crate::app::novel::{Novel, NovelContentAmount, NovelSettings, NovelStatus, NovelType};
use crate::appop::parsers::{cover_image_file, ParseNovel};
use crate::utils::capitalize_str;
use chrono::{Datelike, Local, NaiveDateTime};
use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::str::FromStr;

pub struct Webnovel {
    pub document: Document,
}

impl Webnovel {
    pub fn new(document: Document) -> Self {
        Self { document }
    }
}

impl ParseNovel for Webnovel {
    fn parse_novel(&self, slug: &str) -> Option<Novel> {
        let novel_title = self.parse_title();
        let novel_id = self.generate_id(&novel_title);
        let image = self.parse_image(&novel_id);

        let content = NovelContentAmount {
            chapters: self.parse_chapters(&[]) as f32,
            side_stories: self.parse_side_stories(&[]),
            volumes: self.parse_volumes(&[]),
        };

        let status_string = self
            .document
            .select(Class("det-hd-detail").descendant(Name("strong")))
            .next()
            .unwrap()
            .text()
            .trim()
            .to_string();

        let status = if status_string == "Completed" {
            NovelStatus::Completed
        } else {
            NovelStatus::Ongoing
        };

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
            status,
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
        let meta_keywords = self
            .document
            .select(Attr("name", "keywords"))
            .next()
            .unwrap()
            .attr("content")
            .unwrap()
            .split(',')
            .map(|t| t.trim().to_string())
            .collect::<Vec<String>>();

        meta_keywords[0].clone()
    }

    fn parse_image(&self, novel_id: &str) -> Vec<String> {
        let mut image_url = self
            .document
            .select(Class("g_thumb").descendant(Name("img")))
            .next()
            .unwrap()
            .attr("src")
            .unwrap()
            .to_string();

        let first_two = image_url.chars().take(2).collect::<String>();
        if first_two == "//" {
            // image_url = remove_first_char(image_url);
            // image_url = remove_first_char(image_url);

            image_url = image_url.replace("//", "https://");
        }

        // Get the cover image if there is one available
        if !image_url.contains("nocover") {
            let image_name = sanitize_filename::sanitize(&novel_id);
            let image_path = cover_image_file(&image_url, image_name.as_str());
            return vec![image_path];
        }

        vec![]
    }

    fn parse_description(&self) -> String {
        let novel_description_vec = self
            .document
            .select(Class("j_synopsis").descendant(Name("p")))
            .next()
            .unwrap()
            .inner_html()
            .replace('\n', "")
            .replace("<br>", "\n")
            .trim()
            .split(' ')
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        novel_description_vec.join(" ")
    }

    fn parse_author(&self) -> Vec<String> {
        let meta_keywords = self
            .document
            .select(Attr("property", "og:title"))
            .next()
            .unwrap()
            .attr("content")
            .unwrap()
            .split('-')
            .map(|t| t.trim().to_string())
            .collect::<Vec<String>>();

        let author = meta_keywords[1].clone();
        vec![author]
    }

    fn parse_genre(&self) -> Vec<String> {
        let genre = self
            .document
            .select(Class("det-hd-detail").descendant(Name("a")))
            .next()
            .unwrap()
            .attr("title")
            .unwrap_or("Unknown")
            .to_string();

        vec![genre]
    }

    fn parse_tags(&self) -> Vec<String> {
        self.document
            .select(Class("m-tags").descendant(Name("p")))
            .into_iter()
            .map(|node| capitalize_str(node.text().replace('#', "").trim()))
            .collect()
    }

    fn parse_type(&self) -> NovelType {
        NovelType::from_str("Web Novel").unwrap()
    }

    fn parse_original_language(&self) -> String {
        "English".to_string()
    }

    fn parse_chapters(&self, _strings: &[&str]) -> i32 {
        let strings = self
            .document
            .select(Class("det-hd-detail"))
            .next()
            .unwrap()
            .text()
            .split(' ')
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();

        // Check if the `Vec` of `String`s contains "Chapters"
        // If then assume the previous item has the amount of chapters
        if strings.contains(&"Chapters".to_string()) {
            println!("===== {:?}", strings);
            return strings
                .get(strings.iter().position(|s| s == "Chapters").unwrap() - 1)
                .unwrap()
                .replace(',', "")
                .parse::<i32>()
                .unwrap();
        }

        0
    }

    fn parse_year(&self) -> i32 {
        let head_text = self.document.select(Name("head")).next().unwrap().text();
        let re = Regex::new(r#"datePublished.*"(.*?)Z"#).unwrap();
        let re_captures = re.captures(&head_text);
        if let Some(captures) = re_captures {
            let dt_str = captures.get(1).unwrap().as_str();
            match NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M:%S%.3f") {
                Ok(dt) => {
                    return dt.year();
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }

        0
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
    /// and rename it to be `royalroad.htm` and put it in the project root directory.
    #[test]
    fn test_parser() {
        let test_file = "webnovel.htm";

        if !Path::new(test_file).exists() {
            return;
        }

        let file = fs::read_to_string(test_file).expect("Unable to read file");
        let document = Document::from(file.as_str());

        let novel = NovelParser::Webnovel.parse(document, "https://www.webnovel.com");

        println!("{:?}", novel);

        assert!(novel.is_some());
    }
}

use crate::app::novel::{Novel, NovelContentAmount, NovelSettings, NovelStatus, NovelType};
use crate::appop::parsers::{cover_image_file, numeric_from_str, ParseNovel};
use chrono::Local;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::str::FromStr;

pub struct NovelUpdates {
    document: Document,
}

impl NovelUpdates {
    pub fn new(document: Document) -> Self {
        Self { document }
    }
}

impl ParseNovel for NovelUpdates {
    fn parse_novel(&self, slug: &str) -> Option<Novel> {
        let content_exists = self.document.select(Class("l-content")).next().is_some();
        if !content_exists {
            error!("Could not parse novel from slug: {}", slug);
            return None;
        }

        let novel_title = self.parse_title();
        let novel_id = self.generate_id(&novel_title);
        let image = self.parse_image(&novel_id);

        let mut novel_status = NovelStatus::Other;
        let mut content = NovelContentAmount::default();
        let status_content = self
            .document
            .select(Attr("id", "editstatus"))
            .next()
            .unwrap()
            .text()
            .replace('\n', " ");
        if status_content.trim() != "N/A" {
            let mut split_status: Vec<&str> = status_content.trim().split(' ').collect();
            // Reverse the split list so that any fancy chapter math is at the end
            // which allows this logic to get the correct value(s) instead of the total
            split_status.reverse();

            content.chapters = self.parse_chapters(&split_status) as f32;
            content.side_stories = self.parse_side_stories(&split_status);
            content.volumes = self.parse_volumes(&split_status);
            novel_status = self.parse_status(&split_status);
        }

        let novel = Novel {
            id: novel_id,
            title: novel_title,
            image,
            alternative_titles: Some(self.parse_alt_titles()),
            description: Some(self.parse_description()),
            author: self.parse_author(),
            artist: self.parse_artist(),
            genre: self.parse_genre(),
            tags: self.parse_tags(),
            novel_type: self.parse_type(),
            original_language: self.parse_original_language(),
            translated: self.parse_translated(),
            content,
            status: novel_status,
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
        self.document.select(Class("seriestitlenu")).next().unwrap().text()
    }

    fn parse_image(&self, novel_id: &str) -> Vec<String> {
        let image_url = self
            .document
            .select(Class("seriesimg").descendant(Name("img")))
            .next()
            .unwrap()
            .attr("src")
            .unwrap();

        // Get the cover image if there is one available
        if !image_url.contains("noimagefound") {
            let image_name = sanitize_filename::sanitize(&novel_id);
            let img = cover_image_file(image_url, image_name.as_str());

            return vec![img];
        }

        vec![]
    }

    fn parse_alt_titles(&self) -> Vec<String> {
        self.document
            .select(Attr("id", "editassociated"))
            .next()
            .unwrap()
            .inner_html()
            .split("<br>")
            .map(|s| s.trim().to_string())
            .collect()
    }

    fn parse_description(&self) -> String {
        // \n -> \n\n = paragraph magic
        self.document
            .select(Attr("id", "editdescription"))
            .next()
            .unwrap()
            .text()
            .trim()
            .replace('\n', "\n\n")
    }

    fn parse_author(&self) -> Vec<String> {
        self.document
            .select(Attr("id", "showauthors").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text())
            .collect()
    }

    fn parse_artist(&self) -> Vec<String> {
        self.document
            .select(Attr("id", "showartists").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text())
            .collect()
    }

    fn parse_genre(&self) -> Vec<String> {
        self.document
            .select(Attr("id", "seriesgenre").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text())
            .collect()
    }

    fn parse_tags(&self) -> Vec<String> {
        self.document
            .select(Attr("id", "showtags").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text())
            .collect()
    }

    fn parse_type(&self) -> NovelType {
        let novel_type = self.document.select(Attr("id", "showtype")).next().unwrap().text();

        NovelType::from_str(&novel_type).unwrap()
    }

    fn parse_original_language(&self) -> String {
        self.document
            .select(Attr("id", "showlang").descendant(Name("a")))
            .next()
            .unwrap()
            .text()
            .replace('\n', "")
    }

    fn parse_translated(&self) -> Option<bool> {
        if self
            .document
            .select(Attr("id", "showtranslated"))
            .next()
            .unwrap()
            .text()
            .to_lowercase()
            .contains("yes")
        {
            return Some(true);
        }

        Some(false)
    }

    fn parse_chapters(&self, strings: &[&str]) -> i32 {
        let chapter_strings = ["chapter", "wn chapters"];

        let mut mad_math = false;
        // Collect potential chapter numbers in a vector
        let mut potential_chapters = vec![];
        for target in chapter_strings {
            if let Some(index) = strings.iter().position(|r| r.to_lowercase().contains(&target)) {
                let mut n = 1_usize;
                while strings.len() > index + n {
                    if let Ok(chapters) = numeric_from_str::<i32>(strings[index + n]) {
                        potential_chapters.push(chapters);
                        // In case of "mad math", e.g:
                        // 5 Chapters:
                        // -3 Chapters
                        // -2 Side Stories
                        if strings[index + n].contains(&"-") {
                            mad_math = true;
                        }
                    }
                    n += 1;
                }
            }
        }

        // Fallback to 0
        if potential_chapters.is_empty() {
            return 0;
        }

        // Sort the list so the largest number is last
        potential_chapters.sort_unstable();

        if mad_math {
            // Remove the last element because it is probably the sum of different chapter types
            potential_chapters.remove(potential_chapters.len() - 1);
        }

        // Use the last item as chapter number
        potential_chapters[potential_chapters.len() - 1]
    }

    fn parse_side_stories(&self, strings: &[&str]) -> i32 {
        let side_story_strings = ["side", "special"];

        let mut side_stories = 0;
        let mut added_items = vec![];
        for target in side_story_strings {
            if let Some(index) = strings.iter().position(|r| r.to_lowercase().contains(&target)) {
                let mut n = 1_usize;
                while strings.len() > index + n {
                    if let Ok(sides) = numeric_from_str::<i32>(strings[index + n]) {
                        let pos_in_strings = strings
                            .iter()
                            .position(|s| s.contains(&sides.to_string().as_str()))
                            .unwrap();
                        if !added_items.contains(&pos_in_strings) {
                            // Some may have like:
                            // + 1 extra
                            // 5 specials
                            // etc.
                            // so just sum those together
                            side_stories += sides;
                            added_items.push(pos_in_strings);
                            break;
                        }
                    }
                    n += 1;
                }
            }
        }

        side_stories
    }

    fn parse_volumes(&self, strings: &[&str]) -> i32 {
        let volume_strings = ["volume", "ln volume", "wn volume"];

        for target in volume_strings {
            if let Some(index) = strings.iter().position(|r| r.to_lowercase().contains(&target)) {
                let mut n = 1_usize;
                // Look through all the strings till a match is found
                while strings.len() > index + n {
                    if let Ok(volumes) = numeric_from_str::<i32>(strings[index + n]) {
                        return volumes;
                    }
                    n += 1;
                }
            }
        }

        0
    }

    fn parse_year(&self) -> i32 {
        self.document
            .select(Attr("id", "edityear"))
            .next()
            .unwrap()
            .text()
            .replace('\n', "")
            .parse::<i32>()
            .unwrap_or(0000)
    }

    fn parse_original_publisher(&self) -> Vec<String> {
        self.document
            .select(Attr("id", "showopublisher").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text().replace('\n', ""))
            .collect()
    }

    fn parse_english_publisher(&self) -> Vec<String> {
        self.document
            .select(Attr("id", "showepublisher").descendant(Name("a")))
            .into_iter()
            .map(|node| node.text().replace('\n', ""))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Download any novel page as htm from www.novelupdates.com
    /// and rename it to be `novelupdates.htm` and put it in the project root directory.
    #[test]
    fn test_parser() {
        let test_file = "novelupdates.htm";

        if !Path::new(test_file).exists() {
            return;
        }

        let file = fs::read_to_string(test_file).expect("Unable to read file");
        let document = Document::from(file.as_str());

        let nu_parser = NovelUpdates::new(document);
        let novel = nu_parser.parse_novel("https://www.novelupdates.com");

        println!("{:?}", novel);

        assert!(novel.is_some());
    }
}

use crate::appop::parsers::NovelParser;
use crate::ui::novel_list::ListStatus;
use crate::utils::Resources;
use chrono::prelude::*;
use gdk_pixbuf::Pixbuf;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::io::Cursor;
use std::ops::Index;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct ChapterRead {
    pub volume: i32,
    pub chapter: f32,
    pub side: i32,
    pub exact_num: bool,
    pub novel: Novel,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[repr(i32)]
pub enum NovelStatus {
    Ongoing = 0,
    OriginalCompleted,
    Completed,
    Hiatus,
    Abandoned,
    Other,
}

impl FromStr for NovelStatus {
    type Err = ();

    fn from_str(input: &str) -> Result<NovelStatus, Self::Err> {
        match input {
            "Ongoing" => Ok(NovelStatus::Ongoing),
            "Original completed" => Ok(NovelStatus::OriginalCompleted),
            "Completed" => Ok(NovelStatus::Completed),
            "Hiatus" => Ok(NovelStatus::Hiatus),
            "Abandoned" => Ok(NovelStatus::Abandoned),
            _ => Ok(NovelStatus::Other),
        }
    }
}

impl ToString for NovelStatus {
    fn to_string(&self) -> String {
        match self {
            NovelStatus::Ongoing => fl!("ongoing"),
            NovelStatus::OriginalCompleted => fl!("original-completed"),
            NovelStatus::Completed => fl!("completed"),
            NovelStatus::Hiatus => fl!("hiatus"),
            NovelStatus::Abandoned => fl!("abandoned"),
            NovelStatus::Other => fl!("other"),
        }
    }
}

impl NovelStatus {
    /// Use `to_string` for any visible strings.
    /// For now this is only used for icon names.
    pub fn to_str(&self) -> &'static str {
        match self {
            NovelStatus::Ongoing => "ongoing",
            NovelStatus::OriginalCompleted => "original_completed",
            NovelStatus::Completed => "completed",
            NovelStatus::Hiatus => "hiatus",
            NovelStatus::Abandoned => "abandoned",
            NovelStatus::Other => "other",
        }
    }

    pub fn vec() -> Vec<String> {
        vec![
            NovelStatus::Ongoing.to_string(),
            NovelStatus::OriginalCompleted.to_string(),
            NovelStatus::Completed.to_string(),
            NovelStatus::Hiatus.to_string(),
            NovelStatus::Abandoned.to_string(),
            NovelStatus::Other.to_string(),
        ]
    }

    pub fn to_i32(&self) -> i32 {
        self.to_owned() as i32
    }

    pub fn from_i32(value: i32) -> NovelStatus {
        match value {
            0 => NovelStatus::Ongoing,
            1 => NovelStatus::OriginalCompleted,
            2 => NovelStatus::Completed,
            3 => NovelStatus::Hiatus,
            4 => NovelStatus::Abandoned,
            _ => NovelStatus::Other,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[repr(i32)]
pub enum NovelType {
    WebNovel = 0,
    LightNovel,
    Other,
}

impl FromStr for NovelType {
    type Err = ();

    fn from_str(input: &str) -> Result<NovelType, Self::Err> {
        if input.contains("Web Novel") {
            return Ok(NovelType::WebNovel);
        } else if input.contains("Light Novel") {
            return Ok(NovelType::LightNovel);
        }

        Ok(NovelType::Other)
    }
}

impl ToString for NovelType {
    fn to_string(&self) -> String {
        match self {
            NovelType::WebNovel => fl!("type-web-novel"),
            NovelType::LightNovel => fl!("type-light-novel"),
            NovelType::Other => fl!("type-other"),
        }
    }
}

impl NovelType {
    pub fn vec() -> Vec<String> {
        vec![
            NovelType::WebNovel.to_string(),
            NovelType::LightNovel.to_string(),
            NovelType::Other.to_string(),
        ]
    }

    pub fn from_i32(value: i32) -> NovelType {
        match value {
            0 => NovelType::WebNovel,
            1 => NovelType::LightNovel,
            _ => NovelType::Other,
        }
    }

    pub fn to_i32(&self) -> i32 {
        self.to_owned() as i32
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct NovelContentAmount {
    /// Simply the number of volumes. Both WN and LN can have these.
    pub volumes: i32,
    /// Number of chapters. LNs probably do not have chapters, just volumes and pages.
    /// Float for part numbers; chapter 12.2 for example = chapter 12 part 2.
    pub chapters: f32,
    /// Amount of side stories / specials
    pub side_stories: i32,
}

impl NovelContentAmount {
    pub fn new(volumes: i32, chapters: f32, side_stories: i32) -> Self {
        NovelContentAmount {
            volumes,
            chapters,
            side_stories,
        }
    }

    pub fn from_string(value: String) -> NovelContentAmount {
        lazy_static! {
            static ref RE_VOL: Regex = Regex::new(r"v(\d+)").unwrap();
            static ref RE_CH: Regex = Regex::new(r"c(\d+(?:\.\d+)?)").unwrap();
            static ref RE_SS: Regex = Regex::new(r"ss(\d+)").unwrap();
        }

        let value_lower = value.to_lowercase();

        let vol_captures = RE_VOL.captures(&value_lower);
        let ch_captures = RE_CH.captures(&value_lower);
        let ss_captures = RE_SS.captures(&value_lower);

        let volume = if let Some(capture) = vol_captures {
            capture.get(1).unwrap().as_str().parse().unwrap_or(0)
        } else {
            0
        };

        let chapter = if let Some(capture) = ch_captures {
            capture.get(1).unwrap().as_str().parse().unwrap_or(0.0)
        } else {
            0.0
        };

        let side = if let Some(capture) = ss_captures {
            capture.get(1).unwrap().as_str().parse().unwrap_or(0)
        } else {
            0
        };

        NovelContentAmount {
            volumes: volume,
            chapters: chapter,
            side_stories: side,
        }
    }

    pub fn to_string(&self, pretty: bool) -> String {
        let vol_string = if self.volumes > 0 {
            format!("v{}", self.volumes)
        } else {
            "".to_string()
        };

        let ch_string = if self.chapters > 0.0 {
            format!("c{}", self.chapters)
        } else {
            "".to_string()
        };

        let side_string = if self.side_stories > 0 {
            format!("ss{}", self.side_stories)
        } else {
            "".to_string()
        };

        if pretty {
            format!("{} {} {}", vol_string, ch_string, side_string)
        } else {
            format!("{}{}{}", vol_string, ch_string, side_string)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Novel {
    /// String identifier derived from `title`.
    /// Used for finding the correct novel and for image file names.
    pub id: String,
    /// Name in English.
    pub title: String,
    /// Cover image. Uses the first one in the vector as the cover image.
    /// Using a `Vec` instead of `Option` for potential future uses.
    pub image: Vec<String>,
    /// Name in the original or other languages.
    pub alternative_titles: Option<Vec<String>>,
    /// Description/Synopsis.
    pub description: Option<String>,
    /// Name of one or more authors.
    pub author: Vec<String>,
    /// Name of one or more artists.
    pub artist: Vec<String>,
    /// List of genres
    pub genre: Vec<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Novel types.
    pub novel_type: NovelType,
    /// Country of origin / original language
    pub original_language: String,
    /// If the novel completely translated.
    pub translated: Option<bool>,
    /// Available content in the original language; volumes, chapters and parts.
    pub content: NovelContentAmount,
    /// Novel status.
    pub status: NovelStatus,
    /// The year when the novel saw light in the country of origin.
    pub year: i32,
    /// Name of the original publisher(s).
    pub original_publisher: Vec<String>,
    /// Name of the English publisher(s).
    pub english_publisher: Vec<String>,
    /// Name of the source for all this information.
    pub source: Option<String>,
    /// Slug/Url to the source.
    pub slug: Option<String>,
    /// When novel information was last updated from the source url.
    pub last_scrape: i64,
    /// Local settings for the novel.
    pub settings: NovelSettings,
}

impl Novel {
    /// Novel title
    ///
    /// Adds a prefix `[R]` if rereading is true for this novel,
    /// otherwise does not edit the title.
    pub fn title(&self) -> String {
        // Add a prefix if rereading is set true
        if self.settings.rereading {
            return format!("[R] {}", self.title);
        }

        self.title.to_string()
    }

    /// Returns the correct icon `Pixbuf` that matches the novel `status`.
    pub fn status_pix(&self) -> Pixbuf {
        let file = format!("icons/{}.png", self.status.to_str());
        let resource = Resources::get(&file).unwrap().data;
        Pixbuf::from_read(Cursor::new(resource)).expect("Cannot load pixbuf from resource.")
    }

    /// Shorten original language names.
    ///
    /// E.g: English -> EN
    pub fn orig_lang(&self) -> String {
        let lang = match self.original_language.as_str() {
            "English" => "EN",
            "Korean" => "KR",
            "Japanese" => "JP",
            "Chinese" => "CH",
            _ => "?",
        };

        lang.to_string()
    }

    /// Translated string to when the novel is complete. Since it cannot
    /// be completely translated if the original is not even done.
    pub fn translated(&self) -> Option<String> {
        if let Some(translated) = self.translated {
            return if translated {
                Some(fl!("fully-translated"))
            } else {
                Some(fl!("not-fully-translated"))
            };
        }

        None
    }

    /// Returns the read chapter count as string instead of float with one decimal.
    pub fn chapters_read_str(&self) -> String {
        if self.settings.content_read.chapters.fract() == 0.0 {
            return format!("{:.0}", self.settings.content_read.chapters);
        }

        format!("{:.1}", self.settings.content_read.chapters)
    }

    /// Check if the slug is supported
    pub fn is_slug_supported(&self) -> bool {
        if let Some(slug) = self.slug.clone() {
            return NovelParser::from_url(&slug).is_supported();
        }

        false
    }

    /// Returns a nice string with the available novel content.
    ///
    /// e.g: 3 volumes / 12 chapters
    ///      132 chapters & 3 side stories
    pub fn content(&self) -> String {
        let mut content = String::new();

        if self.content.volumes > 0 {
            let _ = write!(content, "{} ", self.content.volumes);
            let _ = write!(content, "{}", &fl!("volumes").to_lowercase());
        }
        if self.content.chapters > 0.0 {
            if self.content.volumes > 0 {
                content.push_str(" / ");
            }
            let _ = write!(content, "{} ", self.content.chapters);
            let _ = write!(content, "{}", &fl!("chapters").to_lowercase());
        }
        if self.content.side_stories > 0 {
            content.push_str(" & ");
            let _ = write!(content, "{} ", self.content.side_stories);
            let _ = write!(content, "{}", &fl!("side-stories").to_lowercase());
        }

        content
    }

    /// List of authors as `String`.
    pub fn authors(&self) -> String {
        vec_to_string(&self.author)
    }

    /// List of artists as `String`.
    pub fn artists(&self) -> String {
        vec_to_string(&self.artist)
    }

    /// List of genres as `String`.
    pub fn genres(&self) -> String {
        vec_to_string(&self.genre)
    }

    /// List of tags as `String`.
    pub fn tags(&self) -> String {
        vec_to_string(&self.tags)
    }

    /// List of original publishers as `String`.
    pub fn original_publishers(&self) -> String {
        vec_to_string(&self.original_publisher)
    }

    /// List of english publishers as `String`.
    pub fn english_publishers(&self) -> String {
        vec_to_string(&self.english_publisher)
    }

    /// `last_scrape` time as human readable `String`.
    pub fn last_scrape_string(&self) -> String {
        if self.last_scrape == 0 {
            return "Never".to_string();
        }

        let dt = Utc.timestamp(self.last_scrape, 0);
        return dt.format("%d %B %Y, %H:%M:%S").to_string();
    }

    /// If the `Novel` has `slug` then open it in browser.
    pub fn open_slug(&self) {
        if let Some(slug) = self.slug.clone() {
            if webbrowser::open(&slug).is_ok() {}
        }
    }
}

/// Joins a list of Strings into one `String`.
fn vec_to_string(vec: &[String]) -> String {
    vec.join(", ")
}

/// Novel specific settings for each `Novel`.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NovelSettings {
    /// Current list.
    pub list_status: ListStatus,
    /// Data for volumes/chapters/parts read.
    pub content_read: NovelContentAmount,
    /// Personal scoring.
    pub score: String,
    /// Are you reading this novel again?
    pub rereading: bool,
    /// Url which to open for reading.
    pub reading_url: Option<String>,
    /// Keywords that recognition looks for from window titles.
    pub window_titles: Option<Vec<String>>,
    /// Epub file for reading.
    pub file: Option<PathBuf>,
    /// Personal notes.
    pub notes: Option<String>,
    /// Last time `content_read` values changed from the novel recognition system.
    /// Ignores manually changed read counts. It makes more sense this way instead
    /// of updating the value every time the read count changes. Makes managing the
    /// existing novels much nicer.
    pub last_read: i64,
}

impl Default for NovelSettings {
    fn default() -> Self {
        NovelSettings {
            list_status: ListStatus::PlanToRead,
            content_read: NovelContentAmount::default(),
            score: "0.0".to_string(),
            rereading: false,
            reading_url: None,
            window_titles: None,
            file: None,
            notes: None,
            last_read: Local::now().timestamp(),
        }
    }
}

impl NovelSettings {
    pub fn last_read_string(&self) -> String {
        if self.last_read == 0 {
            return "Never".to_string();
        }

        let dt = Utc.timestamp(self.last_read, 0);
        return dt.format("%d %B %Y, %H:%M:%S").to_string();
    }

    /// Try to build a new url from the given reading url.
    ///
    /// Tries to update the chapter number if the url
    /// contains an `chapter-` or `ch-` url segment.
    ///
    /// Returns the given url if the chapter number couldn't be updated.
    fn build_reading_url(&self, url_string: &str, chapter: i32) -> Option<String> {
        let parsed_url = Url::parse(url_string);
        let mut url = match parsed_url {
            Ok(url) => url,
            Err(_) => return None,
        };

        let mut segments = { url.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap() };

        // Use regex to find potential chapter number in the url string
        lazy_static! {
            static ref RE: Regex = Regex::new(r"-\d+").unwrap();
        }

        let caps = RE.captures(url_string);
        let pattern = if let Some(cap) = caps {
            cap.index(cap.len() - 1).to_string()
        } else {
            "ch-".to_string()
        };

        // Try to find the index for the correct vector item
        // so it can be later changed.
        let index = segments
            .iter()
            .position(|s| s.contains(&"chapter-") || s.contains(&pattern));

        if let Some(idx) = index {
            let chapter_segment = segments[idx];
            let mut chapter_items: Vec<String> = chapter_segment.split('-').map(str::to_string).collect();

            // Update the chapter number
            chapter_items.truncate(chapter_items.len() - 1);
            chapter_items.push(chapter.to_string());

            let chapter_string = chapter_items.join("-");

            // Replace the correct vector item
            let _ = std::mem::replace(&mut segments[idx], &chapter_string);

            // Do some stupid magic to effectively create a new owned copy
            // of the segments vector so it can be later used to
            // extend the stupid url.
            let new_segments = segments
                .join("|")
                .split('|')
                .map(|t| t.trim().to_string())
                .collect::<Vec<String>>();

            // Build the new url
            url.path_segments_mut()
                .map_err(|_| "cannot be base")
                .expect("URL path segment error")
                .clear()
                .extend(new_segments);
        }

        Some(url.to_string())
    }

    /// Try to get a proper reading url if it was set.
    pub fn reading_url(&self) -> Option<String> {
        if let Some(url) = self.reading_url.as_ref() {
            return self.build_reading_url(url, self.content_read.chapters as i32 + 1);
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReadAmount {
    pub read: f64,
    pub available: f64,
}

impl ReadAmount {
    pub fn new(available: f64) -> Self {
        ReadAmount { read: 0.0, available }
    }
}

/// Struct for saving data from epub file which is later
/// used to create a new `Novel`.
#[derive(Debug, PartialEq, Clone)]
pub struct NovelFile {
    pub novel_string_id: String,
    pub novel_title: String,
    pub authors: String,
    pub genres: String,
    pub description: String,
    pub chapters: ReadAmount,
    pub status_list_id: String,
    pub slug: Option<String>,
    /// Cover image data
    pub cover_data: Option<Vec<u8>>,
    /// Cover image extension
    pub cover_ext: Option<String>,
}

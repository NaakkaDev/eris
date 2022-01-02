use regex::Regex;
use select::document::Document;
use std::fs::File;
use std::io;
use std::str::FromStr;
use ureq::Error;

pub mod export_import_json;
mod novelupdates;
mod royalroad;
mod scribblehub;

use crate::app::novel::{Novel, NovelStatus, NovelType};
use crate::{data_dir, DATA_IMAGE_DIR};
pub use novelupdates::NovelUpdates;
pub use royalroad::RoyalRoad;
pub use scribblehub::ScribbleHub;
use url::Url;

/// Supported urls
#[derive(PartialEq, Debug)]
pub enum NovelParser {
    None,
    NovelUpdates,
    RoyalRoad,
    ScribbleHub,
}

impl NovelParser {
    pub fn from_url(url: &str) -> Self {
        let url_parse = Url::parse(url);

        match url_parse {
            Ok(url) => {
                if let Some(domain) = url.domain() {
                    return match domain {
                        "www.novelupdates.com" => NovelParser::NovelUpdates,
                        "www.royalroad.com" => NovelParser::RoyalRoad,
                        "www.scribblehub.com" => NovelParser::ScribbleHub,
                        _ => NovelParser::None,
                    };
                }
            }
            Err(e) => {
                error!("Cannot parse url: {:?}", url);
                error!("{}", e);
            }
        };

        NovelParser::None
    }

    pub fn is_supported(&self) -> bool {
        *self != NovelParser::None
    }

    pub fn to_str(&self) -> &str {
        match self {
            NovelParser::NovelUpdates => "www.novelupdates.com",
            NovelParser::RoyalRoad => "www.royalroad.com",
            NovelParser::ScribbleHub => "www.scribblehub.com",
            NovelParser::None => "None",
        }
    }

    pub fn parse(&self, document: Document, slug: &str) -> Option<Novel> {
        match self {
            NovelParser::NovelUpdates => NovelUpdates::new(document).parse_novel(slug),
            NovelParser::RoyalRoad => RoyalRoad::new(document).parse_novel(slug),
            NovelParser::ScribbleHub => ScribbleHub::new(document).parse_novel(slug),
            _ => None,
        }
    }
}

impl FromStr for NovelParser {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Novel Updates" => Ok(NovelParser::NovelUpdates),
            "Royal Road" => Ok(NovelParser::RoyalRoad),
            "Scribble Hub" => Ok(NovelParser::ScribbleHub),
            _ => Ok(NovelParser::None),
        }
    }
}

pub trait ParseNovel {
    fn parse_novel(&self, slug: &str) -> Option<Novel>;
    fn generate_id(&self, title: &str) -> String {
        novel_title_to_slug(title)
    }
    fn generate_source(&self, slug: &str) -> String {
        // `slug´ is never invalid here so this will never fail
        // You will never get this far with invalid URL
        Url::parse(slug).unwrap().domain().unwrap().to_string()
    }
    fn parse_title(&self) -> String;
    fn parse_image(&self, novel_id: &str) -> Vec<String>;
    fn parse_alt_titles(&self) -> Vec<String> {
        vec![]
    }
    fn parse_description(&self) -> String;
    fn parse_author(&self) -> Vec<String> {
        vec![]
    }
    fn parse_artist(&self) -> Vec<String> {
        vec![]
    }
    fn parse_genre(&self) -> Vec<String> {
        vec![]
    }
    fn parse_tags(&self) -> Vec<String> {
        vec![]
    }
    fn parse_type(&self) -> NovelType {
        NovelType::Other
    }
    fn parse_original_language(&self) -> String;
    fn parse_translated(&self) -> Option<bool> {
        None
    }
    fn parse_chapters(&self, _strings: &[&str]) -> i32 {
        0
    }
    fn parse_side_stories(&self, _strings: &[&str]) -> i32 {
        0
    }
    fn parse_volumes(&self, _strings: &[&str]) -> i32 {
        0
    }
    fn parse_status(&self, strings: &[&str]) -> NovelStatus {
        return if strings
            .iter()
            .any(|&s| s.to_lowercase().contains("complete"))
            && self.parse_translated().is_some()
            && !self.parse_translated().unwrap()
        {
            NovelStatus::OriginalCompleted
        } else if strings
            .iter()
            .any(|&s| s.to_lowercase().contains("complete"))
        {
            NovelStatus::Completed
        } else if strings
            .iter()
            .any(|&s| s.to_lowercase().contains("ongoing"))
        {
            NovelStatus::Ongoing
        } else if strings.iter().any(|&s| s.to_lowercase().contains("hiatus")) {
            NovelStatus::Hiatus
        } else if strings
            .iter()
            .any(|&s| s.to_lowercase().contains("dropped"))
        {
            NovelStatus::Dropped
        } else {
            NovelStatus::Other
        };
    }
    fn parse_year(&self) -> i32;
    fn parse_original_publisher(&self) -> Vec<String> {
        vec![]
    }
    fn parse_english_publisher(&self) -> Vec<String> {
        vec![]
    }
}

/// Get the cover image file path as `String`.
///
/// Tries to download the cover image file if it doesn't exist.
fn cover_image_file(url: &str, file_name: &str) -> String {
    let file_path_str = format!("{}/{}.jpg", DATA_IMAGE_DIR, file_name);
    let file_path = data_dir(&file_path_str);

    // If the image file does not exist then try to download it
    if !file_path.exists() {
        match ureq::get(url).call() {
            Ok(response) => {
                // Save image to file
                debug!("Saving image to path: {:?}", file_path);
                let mut out = File::create(file_path).expect("failed to create file");
                io::copy(&mut response.into_reader(), &mut out).expect("failed to copy content");
            }
            Err(Error::Status(code, response)) => {
                warn!("Image cover download response code: {:?}", code);
                warn!("Image cover download response: {:?}", response);
                /* the server returned an unexpected status
                code (such as 400, 500 etc) */
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    } else {
        debug!("File already exists: {:?}", file_path_str);
    }

    file_path_str
}

fn numeric_from_str<F: FromStr>(value: &str) -> Result<F, <F as FromStr>::Err> {
    let re_num = Regex::new(r"(\d+(?:\.\d+)?)").unwrap();
    let num_captures = re_num.captures(value);

    let numeral = if let Some(capture) = num_captures {
        capture.get(1).unwrap().as_str()
    } else {
        "N/A"
    };

    numeral.parse::<F>()
}

/// Turn a novel title into nice slugified `String` which can be used
/// as an URL and a base for cover image file name.
pub fn novel_title_to_slug(novel_title: &str) -> String {
    // Replace some characters to _ to be later changed into nothing
    let pre_slug: String = novel_title
        .chars()
        .map(|x| match x {
            '\'' => '_',
            '’' => '_',
            ',' => '_',
            _ => x,
        })
        .collect();
    // Slugified string from the novel title
    slug::slugify(&pre_slug.replace("_", ""))
}

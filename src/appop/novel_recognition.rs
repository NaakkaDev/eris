use crate::app::novel::Novel;
use crate::app::settings::{NovelRecognitionSettings, Settings};
use crate::app::AppRuntime;
use crate::appop::AppOp;
use chrono::Local;
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use parking_lot::RwLock;
use regex::Regex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use window_titles::{Connection, ConnectionTrait};

pub const SCHEDULE_SECONDS: u32 = 3;

#[derive(Clone)]
pub struct NovelRecognition {
    pub handle: Arc<RwLock<Option<ScheduleHandle>>>,
}

impl NovelRecognition {
    pub fn new(
        app_runtime: AppRuntime,
        settings: NovelRecognitionSettings,
    ) -> Option<NovelRecognition> {
        if !settings.enable {
            return None;
        }

        let wtitles = settings.title_keywords;
        let ititles = settings.ignore_keywords;
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let mut scheduler = Scheduler::new();
        scheduler.every(SCHEDULE_SECONDS.seconds()).run(move || {
            let mut found_title = None;
            let titles_vec = Connection::new().unwrap().window_titles().unwrap();
            'outer: for title in titles_vec {
                // Ignore titles
                for ititle in &ititles {
                    // If title contains ignore keyword then break
                    // out of the loop and do nothing.
                    if title.to_lowercase().contains(&ititle.to_lowercase()) {
                        break 'outer;
                    }
                }
                for wtitle in &wtitles {
                    if title.to_lowercase().contains(&wtitle.to_lowercase()) {
                        found_title = Some(title);
                        break 'outer;
                    }
                }
            }

            tx.send(found_title).expect("Cannot send message");
        });

        rx.attach(None, glib::clone!(@strong app_runtime => @default-return glib::Continue(false), move |value| {
            app_runtime.update_state_with(move |state| {
                state.reading_recognition(value);
            });

            glib::Continue(true)
        }));

        let thread_handle = scheduler.watch_thread(Duration::from_secs(SCHEDULE_SECONDS as u64));

        Some(NovelRecognition {
            handle: Arc::new(RwLock::new(Some(thread_handle))),
        })
    }

    pub fn is_alive(&self) -> bool {
        self.handle.read().is_some()
    }

    pub fn stop(&mut self) {
        self.handle.write().take();
    }
}

#[derive(Debug, Clone, Default)]
pub struct NovelRecognitionData {
    pub volume: i32,
    pub chapter: f32,
    pub side_story: i32,
    pub chapter_title: Option<String>,
    pub source: String,
    pub reading: bool,
}

impl NovelRecognitionData {
    pub fn new(
        volume: i32,
        chapter: f32,
        side_story: i32,
        chapter_title: Option<String>,
        source: String,
        reading: bool,
    ) -> Self {
        NovelRecognitionData {
            volume,
            chapter,
            side_story,
            chapter_title,
            source,
            reading,
        }
    }
}

impl AppOp {
    /// Turn novel recognition on or off.
    pub fn toggle_novel_recognition(&mut self) {
        let mut setting = self.settings.write();

        setting.novel_recognition.enable = !setting.novel_recognition.enable;

        if setting.novel_recognition.enable {
            self.app_runtime.update_state_with(|state| {
                state.novel_recognition = NovelRecognition::new(
                    state.app_runtime.clone(),
                    state.settings.read().novel_recognition.clone(),
                );
            });
        } else if let Some(mut novel_recognition) = self.novel_recognition.clone() {
            thread::spawn(move || {
                novel_recognition.stop();
            });
        }

        // Write to file
        setting.write_to_file().expect("Cannot write setting");
    }

    /// Restart novel recognition system. Used when updating relevant settings.
    /// `settings` has the newest settings.
    pub fn restart_novel_recognition(&mut self, settings: Settings) {
        if let Some(mut novel_recognition) = self.novel_recognition.take() {
            let handle = thread::spawn(move || {
                novel_recognition.stop();
            });

            // No idea if this thing is needed.
            handle.join().expect("Uh oh!");

            self.novel_recognition =
                NovelRecognition::new(self.app_runtime.clone(), settings.novel_recognition);
        }
    }

    /// Parse and use the found window title.
    ///
    /// Used by a message that is send from another thread.
    ///
    /// Updates UI.
    pub fn reading_recognition(&mut self, window_title: Option<String>) {
        debug!("appop:reading_novel_recognition");

        let split_pattern = [" -"];

        debug!("window_title => {:?}", window_title);

        if let Some(window_title) = clean_window_title_string(window_title) {
            let window_title = window_title.replace("|", "-").replace(".epub", "");

            let pattern_index = split_pattern.iter().position(|&p| window_title.contains(p));

            if pattern_index.is_none() {
                return;
            }

            // Turn the title string into a vector of trimmed strings
            let window_title_str: Vec<&str> = window_title
                .split(split_pattern[pattern_index.unwrap()])
                .map(|t| t.trim())
                .collect();

            debug!("Window title str => {:?}", window_title_str);

            let mut novel_title = "?".to_string();
            let source = extract_source_from_title(&window_title_str);
            // Get potential chapter/side story/volume being read from the title strings
            let data = extract_novel_data_from_title(&window_title_str);
            // Try to find the novel based on all the title strings items in the list
            let mut novel = self.find_novel_from_title(&window_title_str);
            if novel.is_none() {
                // Try to extract the novel title from the title strings
                novel_title = extract_novel_name_from_title(&window_title_str);
                if novel_title != "?" {
                    novel = self.get_by_window_title(&novel_title);
                }
            }

            // Check if the currently set novel is the same one that was found
            // based on the title
            let same_novel = if let Some(found_novel) = &novel {
                let is_true =
                    if let Some(current_novel) = self.currently_reading.novel.read().as_ref() {
                        current_novel.id == found_novel.id
                    } else {
                        false
                    };
                is_true
            } else {
                self.currently_reading.novel.read().as_ref().is_none()
            };

            // If the novel is not the same as the currently saved one then
            // show the reading now view and update the current novel
            if !same_novel {
                if let Some(novel) = novel.clone() {
                    self.currently_reading.novel.write().replace(novel);
                } else {
                    self.currently_reading.novel.write().take();
                }
            }

            // Check if the currently reading title is the same as the current one saved
            // Add "novel found boolean" to the "id" string
            let current_title_id = format!("{}-{}", window_title.clone(), novel.is_some());
            let already_done =
                if let Some(current_title) = self.currently_reading.title.read().as_ref() {
                    current_title == &current_title_id
                } else {
                    false
                };

            // Set currently_reading.title
            self.currently_reading
                .title
                .write()
                .replace(current_title_id);

            // Do nothing if things needed to be done are already done and
            // the currently_reading timestamp was "used"
            // meaning the list data was updated (chapter number)
            if already_done && self.currently_reading.timestamp_used {
                return;
            }

            if let Some(novel) = novel.as_mut() {
                // Found novel
                // Check if currently_reading timestamp is set
                if self.currently_reading.timestamp_exists() {
                    if self
                        .currently_reading
                        .timestamp_spend(self.settings.read().novel_recognition.delay)
                    {
                        self.reading_novel(novel, &data, false);
                        self.currently_reading.timestamp_take();
                    } else {
                        // Delay not "spend" yet
                        debug!(
                            "Seconds left till list update: {:?}",
                            (self.currently_reading.timestamp.read().unwrap()
                                + self.settings.read().novel_recognition.delay)
                                - Local::now().timestamp()
                        );
                    }
                } else {
                    // Set the timestamp since it was not set
                    self.currently_reading.timestamp_set()
                }
            } else {
                // Novel suggestions if novel was not found earlier
                let potential_novels = self.find_potential_novels(&novel_title);
                let keyword = novel_title
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .first()
                    .unwrap()
                    .to_string();
                self.ui
                    .show_potential_novels(keyword, potential_novels, self.app_runtime.clone());
            }

            if already_done {
                return;
            }

            // Update the reading now UI
            self.ui
                .update_reading_now(&novel, &novel_title, &data, &source);

            // Decide if the Reading Now view should be shown
            // If novel is none then it was not found in the db
            if (novel.is_none()
                && self
                    .settings
                    .read()
                    .novel_recognition
                    .when_not_novel_go_to_reading)
                || novel.is_some()
                    && self
                        .settings
                        .read()
                        .novel_recognition
                        .when_novel_go_to_reading
            {
                self.ui.show_reading_now_reading();
            }
        } else {
            //** Nothing is being read, apparently **//

            // Do nothing if currently reading is already set to `None`
            if self.currently_reading.title.read().as_ref().is_none() {
                return;
            }

            // Take the value out of `Option` so it becomes `None`
            self.currently_reading.title.write().take();

            // Since the title was None then set the currently reading title and novel to None also
            self.currently_reading.title = Arc::new(RwLock::new(None));
            self.currently_reading.novel = Arc::new(RwLock::new(None));
            self.currently_reading.timestamp_take();

            self.ui.show_reading_not();
            self.currently_reading();
        }
    }

    /// Push a new keyword into novel settings `window_titles`.
    pub fn update_novel_reading_keyword(&mut self, mut novel: Novel, keyword: String) {
        debug!("Updated novel keywords with {:?}", keyword);

        if let Some(keywords) = novel.settings.window_titles.as_mut() {
            // Update
            keywords.push(keyword);
        } else {
            // Add new
            novel.settings.window_titles = Some(vec![keyword]);
        }

        // Save the changes
        let _ = self.update_novel_in_db(novel.clone());
    }

    fn find_novel_from_title(&self, title_strings: &[&str]) -> Option<Novel> {
        // Check if any title string in the list is an exact match
        // with either a novel title or novels recognition keywords
        for title in title_strings {
            let maybe_novel = self.get_by_window_title(title);
            if maybe_novel.is_some() {
                return maybe_novel;
            }
        }

        None
    }
}

fn is_reading_chapter(strings: &[&str]) -> bool {
    // Assumed minimum amount of strings in the list
    // when reading a chapter
    let strings_len: Vec<(&str, usize)> =
        vec![("WuxiaWorld", 3), ("BoxNovel", 3), ("Royal Road", 4)];

    for (key, value) in strings_len.iter() {
        if strings
            .iter()
            .any(|&s| s.to_lowercase().contains(&key.to_lowercase()))
        {
            return &strings.len() >= value;
        }
    }

    // If above does not return `true` then check if the title has
    // the word `chapter` in it.
    if strings
        .iter()
        .any(|&s| s.to_lowercase().contains("chapter"))
    {
        return true;
    }

    false
}

/// Try to parse the novel name from the split title strings.
///
/// If `found_chapter` is true then use 0 as the position
/// for novel name in `strings`.
fn extract_novel_name_from_title(title_strings: &[&str]) -> String {
    // Vector of tuples for finding the novel name
    // -> Website name, position in the list
    // Sites that have the novel name or such as the first item
    // do not need to be added here
    let novel_name_title_pos: Vec<(&str, usize)> = vec![("Bad Reader", 1), ("Royal Road", 1)];

    // If perhaps reading a chapter then the novel name is likely not the
    // first item in the list, otherwise it likely is
    if is_reading_chapter(title_strings) {
        for (source, pos) in novel_name_title_pos.iter() {
            if title_strings.iter().any(|&s| s.contains(source)) {
                return title_strings[*pos].to_string();
            }
        }
    }

    // Decent default
    // try the first item in the title string
    // if there was no match
    title_strings[0].to_string()
}

fn extract_source_from_title(title_strings: &[&str]) -> String {
    // Position (from end) of the source (website name) based on the browser being used
    let position_by_browser = vec![
        ("firefox", 1),
        ("google", 1),
        ("opera", 1),
        ("microsoft", 2),
        ("brave", 1),
    ];

    for (browser, pos) in position_by_browser.iter() {
        if title_strings
            .iter()
            .any(|&s| s.to_lowercase().contains(browser))
        {
            // If the position is 1 then get the second last item
            // e.g:
            //                       --V--
            // [Foo, Bar, Thing, 12, Source, Browser]
            return title_strings[title_strings.len() - 1 - *pos].to_string();
        }
    }

    // Try to return the last item in the list of strings as source
    if let Some(last) = title_strings.last() {
        return last.to_string();
    }

    "?".to_string()
}

/// Try to get the volume/chapter/part number(s) from the split title.
///
/// It is assumed that the part number never goes above 9. (What kind of
/// novel would have so many parts in a chapter anyway.)
fn extract_novel_data_from_title(strings: &[&str]) -> NovelRecognitionData {
    let mut novel_recognition_data =
        NovelRecognitionData::new(0, 0.0, 0, None, "Source".to_string(), false);

    let volume_strings = ["volume", "book", "vol", "v"];
    let chapter_strings = ["chapter", "ch", "c"];
    let part_strings = ["part", "pt."];
    let side_story_strings = ["extra", "side", "special"];
    let mut ignore_part = false;

    'outer_vol: for title_value in strings {
        //
        // Find volume number
        //
        for keyword in volume_strings.iter() {
            let regex_str = format!(r"{}[^\w]", keyword);
            let regex_params = Regex::new(&regex_str).unwrap();
            let title_lower = title_value.to_lowercase();
            let captures = regex_params.captures(&title_lower);

            if title_value.to_lowercase().contains(keyword)
                && title_value.contains(' ')
                && captures.is_some()
            {
                // `Volume 1 -> ["Volume", "1"]
                let split_value: Vec<&str> = title_value.split(' ').map(|s| s.trim()).collect();

                let index = split_value
                    .iter()
                    .position(|s| s.to_lowercase().contains(keyword))
                    .unwrap();

                // Check if the index is the last item in the array
                let potential_volume_number_index = if index + 1 == split_value.len() {
                    index - 1
                } else {
                    index + 1
                };

                // Remove all non-digit characters from the potential volume number
                let volume_num: String = split_value[potential_volume_number_index]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect();

                let potential_volume_num = volume_num.parse::<i32>();
                if let Ok(vol_num) = potential_volume_num {
                    novel_recognition_data.volume = vol_num;
                    break 'outer_vol;
                }
            } else if title_value.to_lowercase().contains(keyword) && captures.is_some() {
                // `volume2` -> 2
                let num = title_value
                    .chars()
                    .skip_while(|c| !c.is_digit(10))
                    .take_while(|c| c.is_digit(10))
                    .fold(None, |acc, c| {
                        c.to_digit(10).map(|b| acc.unwrap_or(0) * 10 + b)
                    });

                if let Some(volume) = num {
                    novel_recognition_data.volume = volume as i32;
                }
            } else {
            }
        }
    }

    'outer_ch: for title_value in strings {
        //
        // Find chapter number
        //
        for keyword in chapter_strings.iter() {
            let regex_str = format!(r"{}[^\w]", keyword);
            let regex_params = Regex::new(&regex_str).unwrap();
            let title_lower = title_value.to_lowercase();
            let captures = regex_params.captures(&title_lower);
            // If the title string contains any of the strings in `chapter_strings`
            // and the title string also contains a space
            // and regex capture found a match
            if title_value.to_lowercase().contains(keyword)
                && title_value.contains(' ')
                && captures.is_some()
            {
                // `Chapter 1` -> ["Chapter", "1"]
                let split_value: Vec<&str> = title_value.split(' ').map(|s| s.trim()).collect();

                let index = split_value
                    .iter()
                    .position(|s| s.to_lowercase().contains(keyword))
                    .unwrap();

                // Check if the index is the last item in the array
                let potential_chapter_number_index = if index + 1 == split_value.len() {
                    index - 1
                } else {
                    index + 1
                };

                // Check if the potential float last char is .
                // e.g. 12.2.
                let mut potential_float = split_value[potential_chapter_number_index];
                let mut potential_float_chars = potential_float.chars();
                if potential_float_chars.clone().last().unwrap() == '.' {
                    potential_float_chars.next_back();
                    potential_float = potential_float_chars.as_str();
                }

                if potential_float.contains('.') {
                    // If the potential float is fine then it is probably good
                    if let Ok(chapter_num) = potential_float.parse::<f32>() {
                        novel_recognition_data.chapter += chapter_num;
                        ignore_part = true;
                        break 'outer_ch;
                    }
                }

                // Remove all non-digit characters from the potential chapter number
                let chapter_num: String = split_value[potential_chapter_number_index]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect();

                let potential_chapter_num = chapter_num.parse::<f32>();
                if let Ok(chap_num) = potential_chapter_num {
                    novel_recognition_data.chapter += chap_num;
                    break 'outer_ch;
                }
            } else if title_value.to_lowercase().contains(keyword) && captures.is_some() {
                // `chapter123` -> 123

                let num = title_value
                    .chars()
                    .skip_while(|c| !c.is_digit(10))
                    .take_while(|c| c.is_digit(10))
                    .fold(None, |acc, c| {
                        c.to_digit(10).map(|b| acc.unwrap_or(0) * 10 + b)
                    });

                if let Some(chapter) = num {
                    novel_recognition_data.chapter += chapter as f32;
                    break 'outer_ch;
                }
            } else {
                // Try to find the first number
                // let num = title_value
                //     .chars()
                //     .skip_while(|c| !c.is_digit(10))
                //     .take_while(|c| c.is_digit(10))
                //     .fold(None, |acc, c| {
                //         c.to_digit(10).map(|b| acc.unwrap_or(0) * 10 + b)
                //     });
                //
                // if let Some(chapter) = num {
                //     novel_recognition_data.chapter += chapter as f32;
                //     break 'outer_ch;
                // }
            }
        }
    }

    'outer_pt: for title_value in strings {
        //
        // Find part number
        //
        if ignore_part {
            break 'outer_pt;
        }

        for keyword in part_strings.iter() {
            if keyword == &"(" {
                let input_re = Regex::new(r"(\(\d+.)").unwrap();

                // Execute the Regex
                let captures = input_re.captures(title_value).map(|captures| {
                    captures
                        .iter()
                        .skip(1) // Skipping the complete match
                        .flatten()
                        .map(|c| c.as_str()) // Grab the original strings
                        .collect::<Vec<_>>()
                });

                if let Some(capture) = captures {
                    let potential_part_num =
                        capture[0].replace("(", "").replace(")", "").parse::<i32>();

                    if let Ok(part_num) = potential_part_num {
                        novel_recognition_data.chapter += part_num as f32 / 10.0;
                        break 'outer_pt;
                    }
                }
            } else {
                let regex_str = format!(r"{}[^\w]", keyword);
                let regex_params = Regex::new(&regex_str).unwrap();
                let title_lower = title_value.to_lowercase();
                let captures = regex_params.captures(&title_lower);

                // If the title string contains any of the strings in `part_strings`
                // and the title string also contains a space
                if title_value.to_lowercase().contains(keyword)
                    && title_value.contains(' ')
                    && captures.is_some()
                {
                    // `Part 1` -> ["Part", "1"]
                    let split_value: Vec<&str> = title_value.split(' ').map(|s| s.trim()).collect();

                    let index = split_value
                        .iter()
                        .position(|s| s.to_lowercase().contains(keyword))
                        .unwrap();

                    // Check if the index is the last item in the array
                    let potential_part_number_index = if index + 1 == split_value.len() {
                        index - 1
                    } else {
                        index + 1
                    };

                    // Remove all non-digit characters from the potential part number
                    let part_num: String = split_value[potential_part_number_index]
                        .chars()
                        .filter(|c| c.is_digit(10))
                        .collect();

                    let potential_part_num = part_num.parse::<f32>();
                    if let Ok(part_num) = potential_part_num {
                        novel_recognition_data.chapter += part_num as f32 / 10.0;
                        break 'outer_pt;
                    }
                } else if title_value.to_lowercase().contains(keyword) && captures.is_some() {
                    // `part2` -> 2
                    let num = title_value
                        .chars()
                        .skip_while(|c| !c.is_digit(10))
                        .take_while(|c| c.is_digit(10))
                        .fold(None, |acc, c| {
                            c.to_digit(10).map(|b| acc.unwrap_or(0) * 10 + b)
                        });

                    if let Some(part_num) = num {
                        novel_recognition_data.chapter += part_num as f32 / 10.0;
                    }
                } else {
                }
            }
        }
    }

    'outer_side: for title_value in strings {
        //
        // Find side story number
        //
        for keyword in side_story_strings.iter() {
            let regex_str = format!(r"{}[^\w]", keyword);
            let regex_params = Regex::new(&regex_str).unwrap();
            let title_lower = title_value.to_lowercase();
            let captures = regex_params.captures(&title_lower);

            // If the title string contains any of the strings in `side_story_strings`
            // and the title string also contains a space
            if title_value.to_lowercase().contains(keyword)
                && title_value.contains(' ')
                && captures.is_some()
            {
                // `Extra Story 1` -> ["Extra", "Story", "1"]
                let split_value: Vec<&str> = title_value.split(' ').map(|s| s.trim()).collect();

                let index = split_value
                    .iter()
                    .position(|s| s.to_lowercase().contains(keyword))
                    .unwrap();

                // Check if the index is the last item in the array
                let potential_side_story_number_index = if index + 1 == split_value.len() {
                    index - 1
                } else {
                    index + 1
                };

                // Remove all non-digit characters from the potential side story number
                let side_story_num: String = split_value[potential_side_story_number_index]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect();

                let potential_part_num = side_story_num.parse::<i32>();
                if let Ok(side_story_num) = potential_part_num {
                    novel_recognition_data.side_story = side_story_num;
                    break 'outer_side;
                }
            } else if title_value.to_lowercase().contains(keyword) && captures.is_some() {
                // `Extra2` -> 2
                let num = title_value
                    .chars()
                    .skip_while(|c| !c.is_digit(10))
                    .take_while(|c| c.is_digit(10))
                    .fold(None, |acc, c| {
                        c.to_digit(10).map(|b| acc.unwrap_or(0) * 10 + b)
                    });

                if let Some(side_story_num) = num {
                    novel_recognition_data.side_story = side_story_num as i32;
                }
            } else {
            }
        }
    }

    novel_recognition_data.reading = is_reading_chapter(strings);

    //
    // Find novel title
    //
    if novel_recognition_data.reading {
        // Try to set chapter title if assumed as reading
        novel_recognition_data.chapter_title = find_chapter_title(strings);
    }

    // If not reading (probably) then any chapter, etc. numbers
    // above 0 are probably not right so zero them
    if !novel_recognition_data.reading {
        novel_recognition_data.chapter = 0.0;
        novel_recognition_data.side_story = 0;
        novel_recognition_data.volume = 0;
    }

    novel_recognition_data
}

fn find_chapter_title(title_strings: &[&str]) -> Option<String> {
    // Position (from end) of the source (website name) based on the browser being used
    let position_by_source = vec![("Royal Road", 0), ("Scribble Hub", 1)];

    for (source, pos) in position_by_source.iter() {
        if title_strings
            .iter()
            .any(|&s| s.to_lowercase().contains(&source.to_lowercase()))
        {
            // If the position is 1 then get the second item
            // e.g:
            //      --V--
            // [Foo, Bar, Thing, 12, Source, Browser]
            return Some(title_strings[*pos].to_string());
        }
    }

    None
}

fn clean_window_title_string(window_title: Option<String>) -> Option<String> {
    if let Some(window_title) = window_title {
        let cleaned = window_title
            .chars()
            .map(|x| match x {
                '|' => '-',
                '–' => '-', // en dash
                '—' => '-', // em dash
                _ => x,
            })
            .collect();

        return Some(cleaned);
    }

    None
}

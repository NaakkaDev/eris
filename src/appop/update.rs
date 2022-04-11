use crate::app::AppRuntime;
use crate::appop::AppOp;
use crate::{UPDATE_LINK, VERSION};
use std::thread;
use std::time::Duration;
use ureq::{Agent, Error};

impl AppOp {
    /// Check if there is a new update available.
    pub fn check_for_update(&self, app_runtime: &AppRuntime) {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let latest_tag = latest_github_tag();
        if let Some(tag) = latest_tag {
            thread::spawn(move || {
                // Current version string to i32
                let current_version = VERSION.replace('.', "").parse::<i32>().expect("");

                if let Ok(version) = tag
                    .chars()
                    .filter(|c| c.is_numeric())
                    .collect::<String>()
                    .parse::<i32>()
                {
                    // If latest release tag version is lower than the version
                    // in the Cargo.toml file (current version) then it means
                    // that there is a new update available.
                    if version > current_version {
                        tx.send(true).expect("Cannot send message");
                    }
                }
            });

            // Show the update available gui element when receiving a message where the value is `true`,
            // nothing should send a `false` though.
            rx.attach(
                None,
                glib::clone!(@strong app_runtime => @default-return glib::Continue(false), move |value| {
                    if value {
                        app_runtime.update_state_with(move |state| {
                            state.ui.toggle_update_menu(true)
                        });
                    }

                    glib::Continue(true)
                }),
            );
        }
    }
}

/// Check latest released tag name in github.
fn latest_github_tag() -> Option<String> {
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(2))
        .timeout_write(Duration::from_secs(2))
        .build();

    match agent.get(UPDATE_LINK).call() {
        Ok(response) => {
            let json: serde_json::Value = response.into_json().expect("Cannot into json");
            let latest_tag = json["tag_name"].to_string();

            return Some(latest_tag);
        }
        Err(Error::Status(code, _response)) => {
            error!(
                "Could not check latest github tag. Url {} returned {}",
                UPDATE_LINK, code
            );
        }
        Err(_) => {}
    }

    None
}

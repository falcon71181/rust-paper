use serde::{Deserialize, Serialize};
use std::default::Default;
use users::get_current_username;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub save_location: String,
    pub integrity: bool,
}

impl Default for Config {
    fn default() -> Self {
        let username = get_current_username()
            .expect("   Failed to get username")
            .to_str()
            .expect("   Failed to convert username to string")
            .to_string();

        let save_location = format!("/home/{}/Pictures/wall", username);

        Config {
            save_location,
            integrity: true,
        }
    }
}

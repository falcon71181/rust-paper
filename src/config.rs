use serde::{Deserialize, Serialize};
use std::default::Default;

use crate::helper;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub save_location: String,
    pub integrity: bool,
}

impl Default for Config {
    fn default() -> Self {
        let username = helper::get_home_location();

        let save_location = format!("{}/Pictures/wall", username);

        Config {
            save_location,
            integrity: true,
        }
    }
}

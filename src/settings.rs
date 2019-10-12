//! Application's settings.

use std::sync::RwLock;

use config::Config;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    /// Settings singleton.
    pub static ref SETTINGS: RwLock<Config> = {
        let settings: Settings = Config::new().try_into().unwrap();
        RwLock::new(Config::try_from(&settings).unwrap())
    };
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub clock: Clock,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Clock {
    pub resolution: u64,
}

impl Default for Clock {
    fn default() -> Self {
        Clock { resolution: 1920 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        assert_eq!(
            Clock::default().resolution,
            SETTINGS
                .read()
                .unwrap()
                .get::<u64>("clock.resolution")
                .unwrap()
        );
    }

    #[test]
    fn merge_file() {
        let file = config::File::from_str(
            "[clock]\nresolution = 12",
            config::FileFormat::Toml,
        );
        SETTINGS.write().unwrap().merge(file).unwrap();

        assert_eq!(
            Clock::default().resolution,
            SETTINGS
                .read()
                .unwrap()
                .get::<u64>("clock.resolution")
                .unwrap()
        );
    }
}

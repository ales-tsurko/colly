//! Application's settings.

use std::sync::RwLock;

use config::Config;
use lazy_static::lazy_static;

lazy_static! {
    /// Settings singleton.
    pub static ref SETTINGS: RwLock<Config> = {
        let mut config = Config::default();
        config.set_default("clock.resolution", Clock::default().resolution as i64).unwrap();
        RwLock::new(config)
    };
}

#[derive(Debug, Default)]
pub struct Settings {
    pub clock: Clock,
}

#[derive(Debug)]
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
            12,
            SETTINGS
                .read()
                .unwrap()
                .get::<u64>("clock.resolution")
                .unwrap()
        );
    }
}

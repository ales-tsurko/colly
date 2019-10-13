//! Application's settings.

use config::{Config, Environment};
use serde::Deserialize;

const ENV_VAR_PREFIX: &str = "COLLY";

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub clock: Clock,
}

impl Settings {
    pub fn new<T>(source: T) -> Result<Self, config::ConfigError>
    where
        T: config::Source + Send + Sync + 'static,
    {
        let mut conf = Config::new();
        conf.merge(source)?;
        conf.merge(Environment::with_prefix(ENV_VAR_PREFIX).separator("__"))?;
        conf.merge(Environment::with_prefix(ENV_VAR_PREFIX))?;

        conf.try_into()
    }
}

#[derive(Debug, Deserialize)]
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
    fn merge_file() {
        let file = config::File::from_str(
            "[clock]\nresolution = 12",
            config::FileFormat::Toml,
        );
        let settings = Settings::new(file).unwrap();

        assert_eq!(12, settings.clock.resolution,);
    }
}

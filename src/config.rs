use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_home")]
    pub home: String,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    #[serde(default = "default_max_redirects")]
    pub max_redirects: usize,
}

fn default_home() -> String {
    "https://text.npr.org".into()
}

fn default_timeout_secs() -> u64 {
    15
}

fn default_max_redirects() -> usize {
    10
}

impl Default for Config {
    fn default() -> Self {
        Self {
            home: default_home(),
            timeout_secs: default_timeout_secs(),
            max_redirects: default_max_redirects(),
        }
    }
}

pub fn load_config() -> Config {
    let mut cfg = Config::default();

    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("textbrowser").join("config.toml");
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(parsed) = toml::from_str::<Config>(&contents) {
                cfg = parsed;
            }
        }
    }

    if let Ok(home_url) = std::env::var("TEXTBROWSER_HOME") {
        if !home_url.is_empty() {
            cfg.home = home_url;
        }
    }

    cfg
}

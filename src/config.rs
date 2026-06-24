use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    /// Directory where cleaned images are saved when no output path is given on the CLI
    pub output_dir: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        find_config_path()
            .and_then(|p| std::fs::read_to_string(&p).ok())
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Resolve the output path for a given input file.
    /// Explicit CLI output takes priority; falls back to output_dir/<input filename>.
    pub fn resolve_output(&self, input: &str, cli_output: Option<String>) -> Result<String, String> {
        if let Some(out) = cli_output {
            return Ok(out);
        }
        let dir = self.output_dir.as_deref().ok_or_else(|| {
            "No output path given and no output_dir set in config. \
             Pass an output path or set output_dir in ~/.config/film-dust-cleaner/config.toml"
                .to_string()
        })?;
        let filename = PathBuf::from(input)
            .file_name()
            .ok_or("Could not determine filename from input path")?
            .to_string_lossy()
            .to_string();
        Ok(format!("{}/{}", dir.trim_end_matches('/'), filename))
    }
}

fn find_config_path() -> Option<PathBuf> {
    // 1. Local config in current directory
    let local = PathBuf::from("film-dust-cleaner.toml");
    if local.exists() {
        return Some(local);
    }
    // 2. User config directory (~/.config/film-dust-cleaner/config.toml)
    dirs::config_dir().map(|d| d.join("film-dust-cleaner").join("config.toml")).filter(|p| p.exists())
}

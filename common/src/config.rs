use std::{
    env::{self, home_dir},
    fs,
    path::PathBuf,
};

use directories::ProjectDirs;
use lazy_static::lazy_static;
use toml::Table;
use tracing::{info, warn};

use crate::{ParseError, TarsError};

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> =
        env::var(format!("{}_DATA", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref CONFIG_FOLDER: Option<PathBuf> =
        env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
}

pub struct TarsConfig;

impl TarsConfig {
    /// Will return the filepath to a tars config on a machine, returing none if filepath doesnt exist
    pub fn get_file_path() -> Option<PathBuf> {
        let mut config_dir = get_config_dir();

        config_dir.push("config.toml");

        info!("File path: {:#?}", config_dir);

        config_dir.canonicalize().ok()
    }

    pub fn get_toml_table() -> Result<Option<Table>, TarsError> {
        let fp = if let Some(fp) = Self::get_file_path() {
            fp
        } else {
            warn!("Was not able to get filepath for config!");
            return Ok(None);
        };

        let config_as_str = fs::read_to_string(fp)?;

        let table: Table = config_as_str
            .parse()
            .map_err(|_e| TarsError::Parse(ParseError::FailedToParse))?;

        Ok(Some(table))
    }
}

pub fn get_data_dir() -> PathBuf {
    if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    }
}

pub fn get_config_dir() -> PathBuf {
    if let Some(s) = CONFIG_FOLDER.clone() {
        return s;
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = home_dir() {
        let macos_preferred = home.join(".config").join("tars");
        return macos_preferred;
    }

    if let Some(home) = home_dir() {
        let generic = home.join(".config");
        return generic;
    }

    if let Some(proj_dirs) = project_directory() {
        return proj_dirs.config_local_dir().to_path_buf();
    }

    PathBuf::from(".").join(".config")
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "suricodes", env!("CARGO_PKG_NAME"))
}

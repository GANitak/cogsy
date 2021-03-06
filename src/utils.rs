use std::fs::{self, read_to_string, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use toml;
use directories::ProjectDirs;

use chrono::{
    FixedOffset,
    DateTime,
    Local,
    Utc,
};
use cursive::theme::{
    Color::*,
    BaseColor,
    PaletteColor::*,
    {BorderStyle, Palette, Theme}
};
use crate::app::database::query;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub username: String,
    pub token: String,
    pub timezone: f32,
    //colours: ColourScheme,
}

impl Config {
    pub fn load() -> Config {
        toml::from_str(
            &read_to_string(config_file()).unwrap_or_default()
        ).unwrap()
    }
    pub fn timezone() -> FixedOffset {
        let raw_tz = Config::load().timezone;
        if raw_tz < 0.0 {
            FixedOffset::west((-raw_tz * 3600.0) as i32)
        } else {
            FixedOffset::east((raw_tz * 3600.0) as i32)
        }
    }
    pub fn first_init() {
        println!("User information not initialized.");
        println!("Username:");
        let mut username = String::new();
        io::stdin().read_line(&mut username)
            .expect("Oops, could not read line.");
        println!("Token:");
        let mut token = String::new();
        io::stdin().read_line(&mut token)
            .expect("Oops, could not read line.");
        println!("Timezone:");
        let mut timezone_raw = String::new(); let mut timezone: f32;
        loop {
            io::stdin().read_line(&mut timezone_raw)
            .expect("Oops, could not read line.");
            match timezone_raw.trim().parse() {
                Ok(num) => {
                    timezone = num;
                }
                Err(_) => {
                    timezone_raw.clear();
                    println!("Please enter a valid timezone.");
                    continue;
                }
            }
            if timezone > 14.0 || timezone < -12.0 {
                println!("Please enter a valid timezone.")
            } else {
                break;
            }
        }
        let config = Config {
            username: username.trim().to_string(),
            token: token.trim().to_string(),
            timezone: timezone,
        };
        if let Ok(new_config) = toml::to_string(&config) {
            match OpenOptions::new().create(true).write(true).open(&config_file()) {
                Ok(ref mut file) => {file.write(new_config.as_bytes()).unwrap();},
                Err(_) => {panic!("Could not create config file!");}
            }
        }
    }
}

pub fn usernames_match() -> bool {
    Config::load().username 
    == 
    query::profile().unwrap().username
}

pub fn get_utc_now() -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        Local::now().naive_utc(),
        Utc
    )
}

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("rs", "cartoon-raccoon", "cogsy")
        .unwrap_or_else(|| panic!("Invalid home directory."))
}

pub fn config_file() -> PathBuf {
    let dirs = project_dirs();
    let mut cfgfile = PathBuf::from(dirs.config_dir());
    fs::create_dir_all(&cfgfile)
        .unwrap_or_else(|_s| panic!("Could not create config file directory"));
    cfgfile.push("config.toml");
    cfgfile
}

pub fn database_file() -> PathBuf {
    let dirs = project_dirs();
    let mut datafile = PathBuf::from(dirs.data_dir());
    fs::create_dir_all(&datafile)
        .unwrap_or_else(|_s| panic!("Could not create data file directory"));
    datafile.push("cogsy_data.db");
    datafile
}

impl Default for Config {
    fn default() -> Config {
        Config {
            username: String::from("null"),
            token: String::from("null"),
            timezone: 0.0,
        }
    }
}

#[derive(Deserialize)]
pub struct ColourScheme {

}

pub fn palette_gen() -> Palette {
    let mut p = Palette::default();
    p[Background] = TerminalDefault;
    p[Shadow] = TerminalDefault;
    p[View] = TerminalDefault;
    p[Primary] = TerminalDefault;
    p[Secondary] = TerminalDefault;
    p[Tertiary] = TerminalDefault;
    p[TitlePrimary] = TerminalDefault;
    p[Highlight] = TerminalDefault;
    p[HighlightInactive] = TerminalDefault;
    p[HighlightText] = Dark(BaseColor::Yellow);

    return p;
}

pub fn theme_gen() -> Theme {
    let mut t = Theme::default();
    t.shadow = false;
    t.borders = BorderStyle::Simple;
    t.palette = palette_gen();
    return t;
}

#[cfg(test)]
mod tests {
    use dirs::home_dir;
    use super::*;
    use crate::app::App;
    #[test] //it fails lmao but code works correctly
    fn check_config_loads_correctly() {
        let testcfg = Config::load();
        let checkcfg = App::initialize();
        println!("{}", testcfg.token);
        assert_eq!(testcfg.username, checkcfg.user_id);
        assert_eq!(testcfg.timezone, 8.0);
    }

    #[test]
    fn check_filepaths() {
        let mut homedir = home_dir().unwrap();
        homedir.push(".config/cogsy/config.toml");
        assert_eq!(config_file(), homedir);
        homedir = home_dir().unwrap();
        homedir.push(".local/share/cogsy/cogsy_data.db");
        assert_eq!(database_file(), homedir);
    }

    #[test]
    fn check_timezone() {
        assert_eq!(
            Config::timezone(),
            FixedOffset::east(28800)
        )
    }
}
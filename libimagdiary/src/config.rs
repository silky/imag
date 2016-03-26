use std::ops::Deref;

use toml::Value;

use libimagrt::runtime::Runtime;

pub enum DiaryType {
    Monthly,
    Weekly,
    Daily,
    Hourly,
    Minutely,
}

impl From<&str> for DiaryType {

    fn from(s: &str) -> DiaryType {
        match s {
            "monthly"   => DiaryType::Monthly,
            "weekly"    => DiaryType::Weekly,
            "hourly"    => DiaryType::Hourly,
            "minutely"  => DiaryType::Minutely,
            "daily" | _ => DiaryType::Daily,
        }
    }
}

pub fn get_diary_type(rt: &Runtime) -> DiaryType {
    rt.config()
        .map(|config| {
            match config.deref() {
                &Value::Table(ref t) => {
                    t.get("diary")
                        .map(|section| {
                            match section {
                                &Value::Table(ref t) => t.get("type").map(DiaryType::from),
                                _ => {
                                    debug!("Config error, expected 'diary' to have a table");
                                    debug!("Falling back to daily entries");
                                    DiaryType::Daily
                                },
                            }
                        })
                },

                _ => {
                    debug!("Config error, expected config to be a table");
                    debug!("Falling back to daily entries");
                    DiaryType::Daily
                }
            }
        })
        .unwrap_or(DiaryType::Daily)
}

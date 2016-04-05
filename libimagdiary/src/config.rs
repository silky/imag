use std::ops::Deref;

use toml::Value;

use libimagrt::runtime::Runtime;

pub enum DiaryType {
    Monthly,
    Daily,
    Hourly,
    Minutely,
}

impl<'a> From<&'a str> for DiaryType {

    fn from(s: &str) -> DiaryType {
        match s {
            "monthly"   => DiaryType::Monthly,
            "hourly"    => DiaryType::Hourly,
            "minutely"  => DiaryType::Minutely,
            "daily" | _ => DiaryType::Daily,
        }
    }
}

/// Get the diary type from the configuration.
///
/// The configuration must look like this:
///
/// ```yaml
/// [diary]
/// default = "my diary"
///
/// [[diary.diaries]]
/// name = "my diary"
/// type = "daily"
///
/// [[diary.diaries]]
/// name = "my other diary"
/// type = "weekly"
/// ```
///
/// So we have two diaries here, the first one named "my diary" and beeing a daily diary, which says
/// that entries are created on a per-day basis. The second diary is a weekly diary named "my other
/// diary". The former diary is the default diary.
pub fn get_diary_type(rt: &Runtime, diaryname: &String) -> DiaryType {
    #[inline]
    fn fallback(s: &str) -> DiaryType {
        debug!("{}", s);
        debug!("Falling back to daily entries");
        DiaryType::Daily
    }

    extract_diary_settings(rt)
        .map(|section| {
            match section {
                &Value::Table(ref t) => t
                    .get("diaries")
                    .map(|v| {
                        if let Some(diary_types) = get_diary_types(v) {
                            for (name, d_type) in diary_types {
                                if &name == diaryname {
                                    return d_type;
                                }
                            }
                        } // no else, as we return in the iteration or we do the following...

                        info!("Could not find diary type settings for {}", diaryname);
                        fallback("")
                    })
                    .unwrap_or_else(|| fallback("Config error, expected 'diary' to have an Array 'diaries'")),

                _ => fallback("Config error, expected 'diary' to have a table")
            }
        })
        .unwrap_or_else(|| fallback("Config error, expected config to be a table"))
}

type DiaryName = String;

fn extract_diary_settings<'a>(rt: &'a Runtime) -> Option<&'a Value> {
    rt.config()
        .and_then(|config| {
            match config.deref() {
                &Value::Table(ref t) => t.get("diary"),
                _ => None,
            }
        })
}

// Expects a Value::Array which is the "diary.diaries" array
fn get_diary_types(v: &Value) -> Option<Vec<(DiaryName, DiaryType)>> {
    match v {
        &Value::Array(ref a) => {
            let mut v = vec![];
            for elem in a {
                if let Some((name, dtype)) = get_diary_name_and_type_from_tab(elem) {
                    v.push((name, DiaryType::from(dtype)));
                }
            }
            Some(v)
        },
        _ => None,
    }
}

fn get_diary_name_and_type_from_tab(v: &Value) -> Option<(DiaryName, DiaryType)> {
    match v {
        &Value::Table(ref t) => {
            if let Some(&Value::String(ref name)) = t.get("name") {
                if let Some(&Value::String(ref dtype)) = t.get("type") {
                    return Some((name.clone(), DiaryType::from(&dtype[..])));
                }
            }
            None
        }
        _ => None,
    }
}

pub fn get_default_diary_name(rt: &Runtime) -> Option<String> {
    rt.config()
        .and_then(|config| config.deref().lookup("diary.default"))
        .and_then(|value| {
            match value {
                &Value::String(ref s) => Some(s.clone()),
                _                     => None,
            }
        })
}

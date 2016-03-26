use std::ops::Deref;

use toml::Value;

use libimagrt::runtime::Runtime;

pub fn create_daily_entries(rt: &Runtime) -> bool {
    rt.config()
        .and_then(|config| {
            let config = config.deref();
            unimplemented!()
        })
        .unwrap_or(true)
}

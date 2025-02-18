use semver::Version;
use toml::Value;

use libimagstore::store::Entry;

use builtin::header::field_path::FieldPath;
use filter::Filter;

pub struct VersionLt {
    version: Version,
}

impl VersionLt {

    pub fn new(version: Version) -> VersionLt {
        VersionLt { version: version }
    }

}

impl Filter for VersionLt {

    fn filter(&self, e: &Entry) -> bool {
        e.get_header()
            .read("imag.version")
            .map(|val| {
                val.map(|v| {
                    match v {
                        Value::String(s) => {
                            match Version::parse(&s[..]) {
                                Ok(v) => v < self.version,
                                _ => false
                            }
                        },
                        _ => false,
                    }
                })
                .unwrap_or(false)
            })
            .unwrap_or(false)
    }

}



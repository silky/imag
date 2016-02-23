use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::Split;

use clap::ArgMatches;
use semver::Version;
use toml::Value;

use libimagstore::store::EntryHeader;
use libimagrt::runtime::Runtime;
use libimagutil::key_value_split::IntoKeyValue;

pub fn build_entry_path(rt: &Runtime, path_elem: &str) -> PathBuf {
    debug!("Checking path element for version");
    {
        let contains_version = {
            path_elem.split("~")
                .last()
                .map(|version| Version::parse(version).is_ok())
                .unwrap_or(false)
        };

        if !contains_version {
            trace!("Version cannot be parsed inside {:?}", path_elem);
            warn!("Path does not contain version. Will panic now!");
            panic!("No version in path");
        }
    }
    debug!("Version checking succeeded");

    trace!("Building path from {:?}", path_elem);
    let mut path = rt.store().path().clone();

    if path_elem.chars().next() == Some('/') {
        path.push(&path_elem[1..path_elem.len()]);
    } else {
        path.push(path_elem);
    }

    path
}

pub fn build_toml_header(matches: &ArgMatches, mut header: EntryHeader) -> EntryHeader {
    debug!("Building header from cli spec");
    if let Some(headerspecs) = matches.values_of("header") {
        let mut main = header.into();
        let kvs = headerspecs.into_iter()
                            .filter_map(|hs| {
                                trace!("- Processing: '{}'", hs);
                                let kv = String::from(hs).into_kv();
                                trace!("-        got: '{:?}'", kv);
                                kv
                            });
        for tpl in kvs {
            let (key, value) = tpl.into();
            trace!("Splitting: {:?}", key);
            let mut split = key.split(".");
            let current = split.next();
            if current.is_some() {
                insert_key_into(String::from(current.unwrap()), &mut split, value, &mut main);
            }
        }

        trace!("Header = {:?}", main);
        EntryHeader::from(main)
    } else {
        trace!("Header = {:?}", header);
        header
    }
}

fn insert_key_into(current: String,
                   rest_path: &mut Split<&str>,
                   value: String,
                   map: &mut BTreeMap<String, Value>) {
    let next = rest_path.next();

    if next.is_none() {
        trace!("Inserting into {:?} = {:?}", current, value);
        map.insert(current, parse_value(value));
    } else {
        trace!("Inserting into {:?} ... = {:?}", current, value);
        if map.contains_key(&current) {
            match map.get_mut(&current).unwrap() {
                &mut Value::Table(ref mut t) => {
                    insert_key_into(String::from(next.unwrap()), rest_path, value, t);
                },
                _ => unreachable!(),
            }
        } else {
            let mut submap = BTreeMap::new();
            insert_key_into(String::from(next.unwrap()), rest_path, value, &mut submap);
            trace!("Inserting submap = {:?}", submap);
            map.insert(current, Value::Table(submap));
        }
    }
}

fn parse_value(value: String) -> Value {
    use std::str::FromStr;

    fn is_ary(v: &String) -> bool {
        v.chars().next() == Some('[') && v.chars().last() == Some(']') && v.len() >= 3
    }

    if value == "true" {
        trace!("Building Boolean out of: {:?}...", value);
        Value::Boolean(true)
    } else if value == "false" {
        trace!("Building Boolean out of: {:?}...", value);
        Value::Boolean(false)
    } else if is_ary(&value) {
        trace!("Building Array out of: {:?}...", value);
        let sub = &value[1..(value.len()-1)];
        Value::Array(sub.split(",").map(|v| parse_value(String::from(v))).collect())
    } else {
        FromStr::from_str(&value[..])
            .map(|i: i64| {
                trace!("Building Integer out of: {:?}...", value);
                Value::Integer(i)
            })
            .unwrap_or_else(|_| {
                FromStr::from_str(&value[..])
                    .map(|f: f64| {
                        trace!("Building Float out of: {:?}...", value);
                        Value::Float(f)
                    })
                    .unwrap_or_else(|_| {
                        trace!("Building String out of: {:?}...", value);
                        Value::String(value)
                    })
            })
    }
}


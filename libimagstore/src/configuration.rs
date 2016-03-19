use toml::Value;

/// Check whether the configuration is valid for the store
///
/// The passed `Value` _must be_ the `[store]` sub-tree of the configuration. Otherwise this will
/// fail.
///
/// It checks whether the configuration looks like the store wants it to be:
///
/// ```toml
/// [store]
/// pre-create-hook-aspects = [ "misc", "encryption", "version-control"]
///
/// [[aspects.misc]]
/// parallel = true
/// [[aspects.encryption]]
/// parallel = false
/// [[aspects.version-control]]
/// parallel = false
///
/// [[hooks.gnupg]]
/// aspect = "encryption"
/// key = "0x123456789"
///
/// [[hooks.git]]
/// aspect = "version-control"
/// ```
///
/// It checks:
///  * Whether all the maps are there (whether store, store.aspects, store.aspects.example are all
///  maps)
///  * Whether each aspect configuration has a "parallel = <Boolean>" setting
///  * Whether each hook congfiguration has a "aspect = <String>" setting
///
/// It does NOT check:
///  * Whether all aspects which are used in the hook configuration are also configured
///
/// No configuration is a valid configuration, as the store will use the most conservative settings
/// automatically. This has also performance impact, as all hooks run in no-parallel mode then.
/// You have been warned!
///
///
pub fn config_is_valid(config: &Option<Value>) -> bool {
    use std::collections::BTreeMap;
    use std::io::Write;
    use std::io::stderr;

    if config.is_none() {
        return true;
    }

    fn has_key_with_string_ary(v: &BTreeMap<String, Value>, key: &str) -> bool {
        v.get(key)
            .map(|t| match t {
                    &Value::Array(ref a) => a.iter().all(|elem| {
                        match elem {
                            &Value::String(_) => true,
                            _ => false,
                        }
                    }),
                    _ => {
                        write!(stderr(), "Key '{}' in store config should contain an array", key)
                            .ok();
                        false
                    }
            }).unwrap_or_else(|| {
                write!(stderr(), "Required key '{}' is not in store config", key).ok();
                false
            })
    }

    /// Check that
    /// * the top-level configuration
    /// * is a table
    /// * where all entries of a key `section` (eg. "hooks" or "aspects")
    ///     * Are maps
    ///     * where each has a key `key` (eg. "aspect" or "parallel")
    ///     * which fullfills constraint `f` (typecheck)
    fn check_all_inner_maps_have_key_with<F>(store_config: &BTreeMap<String, Value>,
                                             section: &str,
                                             key: &str,
                                             f: F)
        -> bool
        where F: Fn(&Value) -> bool
    {
        store_config.get(section) // The store config has the section `section`
            .map(|section_table| {
                match section_table { // which is
                    &Value::Table(ref section_table) => // a table
                        section_table
                            .iter() // which has values,
                            .all(|(inner_key, cfg)| { // and all of these values
                                match cfg {
                                    &Value::Table(ref hook_config) => { // are tables
                                        hook_config.get(key) // with a key
                                            // fullfilling this constraint
                                            .map(|hook_aspect| f(&hook_aspect))
                                            .unwrap_or(false)
                                    },
                                    _ => {
                                        write!(stderr(), "Store config expects '{}' to be in '{}.{}', but isn't.",
                                                 key, section, inner_key).ok();
                                        false
                                    }
                                }
                            }),
                    _ => {
                        write!(stderr(), "Store config expects '{}' to be a Table, but isn't.",
                               section).ok();
                        false
                    }
                }
            })
            .unwrap_or_else(|| {
                write!(stderr(), "Store config expects section '{}' to be present, but isn't.",
                        section).ok();
                false
            })
    }

    match config {
        &Some(Value::Table(ref t)) => {
            has_key_with_string_ary(t, "pre-read-hook-aspects")        &&
            has_key_with_string_ary(t, "post-read-hook-aspects")       &&
            has_key_with_string_ary(t, "pre-create-hook-aspects")      &&
            has_key_with_string_ary(t, "post-create-hook-aspects")     &&
            has_key_with_string_ary(t, "pre-retrieve-hook-aspects")    &&
            has_key_with_string_ary(t, "post-retrieve-hook-aspects")   &&
            has_key_with_string_ary(t, "pre-update-hook-aspects")      &&
            has_key_with_string_ary(t, "post-update-hook-aspects")     &&
            has_key_with_string_ary(t, "pre-delete-hook-aspects")      &&
            has_key_with_string_ary(t, "post-delete-hook-aspects")     &&

            // The section "hooks" has maps which have a key "aspect" which has a value of type
            // String
            check_all_inner_maps_have_key_with(t, "hooks", "aspect", |asp| {
                let res = match asp { &Value::String(_) => true, _ => false };
                res
            }) &&

            // The section "aspects" has maps which have a key "parllel" which has a value of type
            // Boolean
            check_all_inner_maps_have_key_with(t, "aspects", "parallel", |asp| {
                let res = match asp { &Value::Boolean(_) => true, _ => false, };
                res
            })
        }
        _ => {
            write!(stderr(), "Store config is no table").ok();
            false
        },
    }
}

pub fn get_pre_create_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("pre-create-hook-aspects", value)
}

pub fn get_post_create_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("post-create-hook-aspects", value)
}

pub fn get_pre_retrieve_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("pre-retrieve-hook-aspects", value)
}

pub fn get_post_retrieve_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("post-retrieve-hook-aspects", value)
}

pub fn get_pre_update_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("pre-update-hook-aspects", value)
}

pub fn get_post_update_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("post-update-hook-aspects", value)
}

pub fn get_pre_delete_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("pre-delete-hook-aspects", value)
}

pub fn get_post_delete_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("post-delete-hook-aspects", value)
}

pub fn get_pre_move_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("pre-move-hook-aspects", value)
}

pub fn get_post_move_aspect_names(value: &Option<Value>) -> Vec<String> {
    get_aspect_names_for_aspect_position("post-move-hook-aspects", value)
}

#[derive(Debug)]
pub struct AspectConfig {
    parallel: bool,
    config: Value,
}

impl AspectConfig {

    pub fn new(init: Value) -> AspectConfig {
        let parallel = AspectConfig::is_parallel(&init);
        AspectConfig {
            config: init,
            parallel: parallel,
        }
    }

    fn is_parallel(init: &Value) -> bool {
        match init {
            &Value::Table(ref t) =>
                t.get("parallel")
                    .map(|value| {
                        match value {
                            &Value::Boolean(b) => b,
                            _ => false,
                        }
                    })
                    .unwrap_or(false),
            _ => false,
        }
    }

    /// Get the aspect configuration for an aspect.
    ///
    /// Pass the store configuration object, this searches in `[aspects][<aspect_name>]`.
    ///
    /// Returns `None` if one of the keys in the chain is not available
    pub fn get_for(v: &Option<Value>, a_name: String) -> Option<AspectConfig> {
        match v {
            &Some(Value::Table(ref tabl)) => tabl.get(&a_name[..])
                .map(|asp| AspectConfig::new(asp.clone())),
            _ => None,
        }
    }

}

fn get_aspect_names_for_aspect_position(config_name: &'static str, value: &Option<Value>) -> Vec<String> {
    let mut v = vec![];

    match value {
        &Some(Value::Table(ref t)) => {
            match t.get(config_name) {
                Some(&Value::Array(ref a)) => {
                    for elem in a {
                        match elem {
                            &Value::String(ref s) => v.push(s.clone()),
                            _ => warn!("Non-String in configuration, inside '{}'", config_name),
                        }
                    }
                },
                _ => warn!("'{}' configuration key should contain Array, does not", config_name),
            };
        },
        &None => warn!("No store configuration"),
        _ => warn!("Configuration is not a table"),
    }
    v
}



use std::default::Default;
use std::path::PathBuf;
use std::result::Result as RResult;

pub use config::types::Config;
pub use config::reader::from_file;

/**
 * Errors which are related to configuration-file loading
 */
pub mod error {
    use std::error::Error;
    use std::fmt::{Debug, Display, Formatter};
    use std::fmt::Error as FmtError;

    /**
     * The kind of an error
     */
    #[derive(Clone, Debug, PartialEq)]
    pub enum ConfigErrorKind {
        ConfigNotFound,
        ConfigParsingFailed,
        NoConfigFileFound,
    }

    /**
     * Configuration error type
     */
    #[derive(Debug)]
    pub struct ConfigError {
        kind: ConfigErrorKind,
        cause: Option<Box<Error>>,
    }

    impl ConfigError {

        /**
         * Instantiate a new ConfigError, optionally with cause
         */
        pub fn new(kind: ConfigErrorKind, cause: Option<Box<Error>>) -> ConfigError {
            ConfigError {
                kind: kind,
                cause: cause,
            }
        }

        /**
         * get the Kind of the Error
         */
        pub fn kind(&self) -> ConfigErrorKind {
            self.kind.clone()
        }

        /**
         * Get the string, the ConfigError can be described with
         */
        pub fn as_str(e: &ConfigError) -> &'static str {
            match e.kind() {
                ConfigErrorKind::ConfigNotFound      => "Config not found",
                ConfigErrorKind::ConfigParsingFailed => "Config parsing failed",
                ConfigErrorKind::NoConfigFileFound   => "No config file found",
            }
        }

    }

    impl Display for ConfigError {

        fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
            try!(write!(fmt, "{}", ConfigError::as_str(self)));
            Ok(())
        }

    }

    impl Error for ConfigError {

        fn description(&self) -> &str {
            ConfigError::as_str(self)
        }

        fn cause(&self) -> Option<&Error> {
            self.cause.as_ref().map(|e| &**e)
        }

    }

}

use self::error::{ConfigError, ConfigErrorKind};


/**
 * Result type of this module. Either T or ConfigError
 */
pub type Result<T> = RResult<T, ConfigError>;

/**
 * Configuration object
 *
 * Holds all config variables which are globally available plus the configuration object from the
 * config parser, which can be accessed.
 */
#[derive(Debug)]
pub struct Configuration {

    /**
     * The verbosity the program should run with
     */
    verbosity: bool,

    /**
     * The editor which should be used
     */
    editor: Option<String>,

    /**
     * The options the editor should get when opening some file
     */
    editor_opts: String,
}

impl Configuration {

    /**
     * Get a new configuration object.
     *
     * The passed runtimepath is used for searching the configuration file, whereas several file
     * names are tested. If that does not work, the home directory and the XDG basedir are tested
     * with all variants.
     *
     * If that doesn't work either, an error is returned.
     */
    pub fn new(rtp: &PathBuf) -> Result<Configuration> {
        fetch_config(&rtp).map(|cfg| {
            let verbosity   = cfg.lookup_boolean("verbosity").unwrap_or(false);
            let editor      = cfg.lookup_str("editor").map(String::from);
            let editor_opts = String::from(cfg.lookup_str("editor-opts").unwrap_or(""));

            trace!("Building configuration");
            trace!("  - verbosity  : {:?}", verbosity);
            trace!("  - editor     : {:?}", editor);
            trace!("  - editor-opts: {}", editor_opts);

            Configuration {
                verbosity: verbosity,
                editor: editor,
                editor_opts: editor_opts,
            }
        })
    }

}

/**
 * Helper to fetch the config file
 *
 * Tests several variants for the config file path and uses the first one which works.
 */
fn fetch_config(rtp: &PathBuf) -> Result<Config> {
    use std::env;

    use xdg_basedir;
    use itertools::Itertools;

    use libimagutil::variants::generate_variants as gen_vars;

    let variants = vec!["config", "config.toml", "imagrc", "imagrc.toml"];
    let modifier = |base: &PathBuf, v: &'static str| {
        let mut base = base.clone();
        base.push(format!("/{}", v));
        base
    };

    vec![
        gen_vars(rtp.clone(), variants.clone(), &modifier),

        env::var("HOME").map(|home| gen_vars(PathBuf::from(home), variants.clone(), &modifier))
                        .unwrap_or(vec![]),

        xdg_basedir::get_data_home().map(|data_dir| gen_vars(data_dir, variants.clone(), &modifier))
                                    .unwrap_or(vec![]),
    ].iter()
        .flatten()
        .filter(|path| path.exists())
        .map(|path| {
            from_file(&path)
                    .map_err(|e| {
                        ConfigError::new(ConfigErrorKind::ConfigParsingFailed, Some(Box::new(e)))
                    })
        })
        .filter(|loaded| loaded.is_ok())
        .nth(0)
        .map(|inner| inner.unwrap())
        .ok_or(ConfigError::new(ConfigErrorKind::NoConfigFileFound, None))
}

impl Default for Configuration {

    /**
     * Get a default configuration object
     */
    fn default() -> Configuration {
        Configuration {
            verbosity: false,
            editor: Some(String::from("nano")),
            editor_opts: String::from(""),
        }
    }

}


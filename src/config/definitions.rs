
use std::fs::File;
use std::io::{Read, Error};
use std::path::{PathBuf, Path};
use std::default::Default;
use std::fmt;

use yaml_rust::{Yaml, YamlLoader, ScanError};

use config::FromYaml;

pub enum ConfigError {
    YamlScan(ScanError),
    YamlParse(ConfigMeta),
    FileError(String, Error),
    MissingExternalFile(ConfigMeta),
}

impl ConfigError {
    pub fn to_string(&self) -> String {
        match self {
            &ConfigError::YamlScan(ref e) => format!("Failed to scan YAML: {}", e),
            &ConfigError::YamlParse(ref meta) => {
                let mut path = String::new();

                for (index, element) in meta.fields.iter().enumerate() {
                    if index != 0 {
                        path = path + "->";
                    }

                    path = path + element;
                }

                let message = if meta.bad_value {
                    "Could not find YAML"
                } else {
                    "Failed to parse YAML"
                };

                let basic = format!("{}: {}: {}: expected {}", meta.path.display(), message, path, meta.ty);

                let options = if meta.options.len() > 0 {
                    let mut result = "".to_string();

                    for (index, option) in meta.options.iter().enumerate() {
                        if index != 0 {
                            result = result + ", ";
                        }

                        result = result + option;
                    }

                    format!("\n{}:\t {} {{ {} }}", meta.path.display(), meta.ty, result)
                } else {
                    "".to_string()
                };

                format!("{}{}", basic, options)
            },
            &ConfigError::FileError(ref disp, ref e) => format!("Config File Error: \"{}\", {}", disp, e),
            &ConfigError::MissingExternalFile(ref meta) => format!("{}: External YAML file is missing", meta.path.display()),
        }
    }
}

impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct ConfigMeta {
    pub path: PathBuf, // Where the file is located, "" if not from a file.
    pub fields: Vec<String>, // List from top-level to bottom-level configs
    pub ty: &'static str, // String representation of the type
    pub bad_value: bool, // Whether key is bad or not
    pub options: Vec<String>, // Options to display to user, usually used for enums
}

impl Default for ConfigMeta {
    fn default() -> Self {
        ConfigMeta {
            path: PathBuf::from(""),
            fields: Vec::new(),
            ty: "T",
            bad_value: false,
            options: Vec::new(),
        }
    }
}

pub trait FromFile: Sized {
    // From a file relative to current config
    fn from_file_raw(meta: &ConfigMeta, path: &Path) -> Result<Self, ConfigError>;

    // From a file relative to project
    fn from_file(path: &Path) -> Result<Self, ConfigError> {
        Self::from_file_raw(&ConfigMeta::default(), path)
    }
}

impl<T: FromYaml + Sized> FromFile for T {
    fn from_file_raw(meta: &ConfigMeta, path: &Path) -> Result<T, ConfigError> {
        let mut next_meta = meta.clone();
        let mut field_path = meta.path.parent().unwrap_or(Path::new("")).to_path_buf();

        field_path.push(path);

        if field_path.is_dir() && field_path.exists() {
            field_path.push("config");
        }

        field_path.set_extension("yml");

        // extra check for a file that uses the alternate extensions .yaml instead of .yml
        if !field_path.exists() {
            field_path.set_extension("yaml");
        }

        next_meta.path = field_path;

        if next_meta.path.exists() {
            let mut file = try!(File::open(next_meta.path.as_path())
                .map_err(|e| ConfigError::FileError(next_meta.path.as_path().display().to_string(), e)));
            let mut buffer = String::new();

            try!(file.read_to_string(&mut buffer)
                .map_err(|e| ConfigError::FileError(next_meta.path.as_path().display().to_string(), e)));

            let yaml = try!(YamlLoader::load_from_str(&buffer)
                .map_err(|e| ConfigError::YamlScan(e)));
            let hash = &yaml[0];

            <T>::from_yaml(&next_meta, hash)
        }
        else {
            Err(ConfigError::MissingExternalFile(next_meta.clone()))
        }
    }
}

#[macro_export]
macro_rules! config_enum {
    ($root:ident {
        $( $field:ident, )*
    }) => {

        #[derive(Clone, Debug)]
        pub enum $root {
            $($field,)*
        }

        impl FromYaml for $root {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                let mut next_meta = meta.clone();
                next_meta.options = vec![$( stringify!($field).to_string(), )*];

                if let &Yaml::String(ref string) = config {
                    let s: &str = string;

                    match s {
                        $(
                            stringify!($field) => Ok($root::$field),
                        )*
                        _ => Err(ConfigError::YamlParse(next_meta.clone()))
                    }
                }
                else {
                    Err(ConfigError::YamlParse(next_meta.clone()))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! config {
    ($root:ident {
        $( $field:ident: $ty:ty = $name:expr, )*
    }) => {
        #[derive(Clone, Debug)]
        pub struct $root {
            _meta: ConfigMeta,
            $( pub $field: $ty, )*
        }

        impl Default for $root {
            fn default() -> Self {
                $root {
                    _meta: ConfigMeta::default(),
                    $( $field: $name, )*
                }
            }
        }

        impl FromYaml for $root {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                let mut default = $root::default();

                let mut next_meta = meta.clone();
                next_meta.ty = stringify!($root);

                // Appends top-level
                if meta.fields.len() == 0 {
                    next_meta.fields.push(stringify!($root).to_string());
                }

                default._meta = next_meta.clone();

                Ok($root {
                    _meta: default._meta,
                    $(
                        $field: {
                            let key = &config[stringify!($field)];

                            // set up current meta
                            let mut field_meta = next_meta.clone();
                            field_meta.fields.push(stringify!($field).to_string());
                            field_meta.ty = stringify!($ty);
                            field_meta.bad_value = key.is_badvalue();

                            let val = if key.as_str() == Some("extern") { // external file
                                <$ty>::from_file_raw(&field_meta, Path::new(stringify!($field)))
                            }
                            else { // current file
                                <$ty>::from_yaml(&field_meta, key)
                            };

                            match val {
                                Ok(found) => found,
                                Err(e) => {
                                    // output error and fall-through the default values
                                    println!("{}", e);

                                    default.$field
                                },
                            }
                        },
                    )*
                })
            }
        }
    }
}
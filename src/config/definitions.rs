
//! Configuration structures and macros

use std::io::Error;
use std::path::PathBuf;
use std::default::Default;
use std::fmt;

use yaml_rust::ScanError;

/// Configuration error
pub enum ConfigError {
    YamlScan(ScanError),
    YamlParse(ConfigMeta),
    YamlGeneric(String),
    FileError(String, Error),
    MultipleExternalFiles(String, Vec<PathBuf>),
    MissingExternalFile(ConfigMeta),
}

impl ConfigError {
    pub fn to_string(&self) -> String {
        match self {
            &ConfigError::YamlScan(ref e) => format!("Failed to scan YAML: {}", e),
            &ConfigError::YamlParse(ref meta) => {
                let mut tree = String::new();

                for (index, element) in meta.fields.iter().enumerate() {
                    if index != 0 {
                        tree = tree + "->";
                    }

                    tree = tree + element;
                }

                let message = if meta.bad_value {
                    "Could not find YAML"
                }
                else {
                    "Failed to parse YAML"
                };

                let path = meta.path.display().to_string() + ": ";

                let basic = format!("{}{}: {}: expected {}", path, message, tree, meta.ty);

                let options = if meta.options.len() > 0 {
                    let mut result = "".to_string();

                    for (index, option) in meta.options.iter().enumerate() {
                        if index != 0 {
                            result = result + ", ";
                        }

                        result = result + option;
                    }

                    format!("\n{}:\t {} {{ {} }}", path, meta.ty, result)
                }
                else {
                    "".to_string()
                };

                format!("{}{}", basic, options)
            },
            &ConfigError::YamlGeneric(ref string) => string.clone(),
            &ConfigError::FileError(ref disp, ref e) => format!("Config File Error: \"{}\", {}", disp, e),
            &ConfigError::MultipleExternalFiles(ref name, ref conflicts) => {
                let mut result = "".to_string();

                for (index, conflict) in conflicts.iter().enumerate() {
                    if index != 0 {
                        result = result + ",\n\t";
                    }

                    result = result + &conflict.display().to_string();
                }

                format!("{}: Multiple external files: \n\t{}", name, result)
            },
            &ConfigError::MissingExternalFile(ref meta) => {
                format!("{}: External YAML file is missing", meta.path.display().to_string())
            },
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

/// Metadata for a configuration structure
#[derive(Clone, Debug)]
pub struct ConfigMeta {
    pub path: PathBuf, // Where the file is located, defaults to "config/config.yml"
    pub fields: Vec<String>, // List from top-level to bottom-level configs
    pub ty: &'static str, // String representation of the type
    pub bad_value: bool, // Whether key is bad or not
    pub options: Vec<String>, // Options to display to user, usually used for enums
}

impl Default for ConfigMeta {
    fn default() -> Self {
        ConfigMeta {
            path: PathBuf::from("config\\config.yml"),
            fields: Vec::new(),
            ty: "Unknown Type",
            bad_value: false,
            options: Vec::new(),
        }
    }
}

/// Automatically generates a struct/enums for loading in yaml files.
#[macro_export]
macro_rules! config {
    // Struct
    (
        $(#[$root_meta:meta])*
        struct $root:ident {
            $( $(#[$field_meta:meta])* pub $field:ident: $ty:ty = $name:expr, )*
        }
    ) => {
        #[derive(Clone, Debug)]
        $(#[$root_meta])*
        pub struct $root {
            _meta: ConfigMeta,
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )*
        }

        impl $root {
            pub fn to_string(&self) -> String {
                $crate::config::to_string(&self.to_yaml(&self._meta.path.clone().as_path()))
            }
        }

        impl Default for $root {
            fn default() -> Self {
                $root {
                    _meta: ConfigMeta::default(),
                    $( $field: $name, )*
                }
            }
        }

        impl Element for $root {
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

                                    default.$field.set_meta(&field_meta);

                                    default.$field
                                },
                            }
                        },
                    )*
                })
            }

            fn to_yaml(&self, path: &Path) -> Yaml {
                use std::collections::BTreeMap;

                let mut map: BTreeMap<Yaml, Yaml> = BTreeMap::new();

                $(
                    map.insert(
                        Yaml::String(stringify!($field).to_string()),
                        self.$field.to_yaml(path),
                    );

                    if let Some(field_meta) = self.$field.get_meta() {
                        if field_meta.path != path {
                            map.insert(
                                Yaml::String(stringify!($field).to_string()),
                                Yaml::String("extern".to_string()),
                            );
                        }
                    }
                )*

                Yaml::Hash(map)
            }

            fn set_meta(&mut self, meta: &ConfigMeta) {
                self._meta = meta.clone();
            }

            fn get_meta(&self) -> Option<ConfigMeta> {
                Some(self._meta.clone())
            }

            fn write_file(&self) -> Result<(), ConfigError> {
                use std::fs::{DirBuilder, File};
                use std::io::{Write};

                let path = self._meta.path.clone();
                let readable = $crate::config::to_string(&self.to_yaml(&path.as_path()));

                // Recursively create in the case of new project or deleted folders
                try!(DirBuilder::new().recursive(true).create(&path.parent().unwrap())
                    .map_err(|e| ConfigError::FileError(path.display().to_string(), e)));

                let mut file = try!(File::create(&path)
                    .map_err(|e| ConfigError::FileError(path.display().to_string(), e)));
                try!(file.write_all(readable.as_bytes())
                    .map_err(|e| ConfigError::FileError(path.display().to_string(), e)));

                $(
                    if let Some(ref field_meta) = self.$field.get_meta() {
                        if field_meta.path != path {
                            if let Err(e) = self.$field.write_file() {
                                return Err(e);
                            }
                        }
                    }
                )*

                Ok(())
            }
        }
    };

    // Enum
    (
        $(#[$root_meta:meta])*
        enum $root:ident {
            $( $field:ident, )*
        }
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        $(#[$root_meta])*
        pub enum $root {
            $($field,)*
        }

        impl Element for $root {
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

            fn to_yaml(&self, _: &Path) -> Yaml {
                match self {
                    $(
                        &$root::$field => Yaml::String(stringify!($field).to_string()),
                    )*
                }
            }
        }
    };
}
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
    NonConfig,
    FileError(PathBuf, Error),
    MultipleExternalFiles(PathBuf, Vec<PathBuf>),
    MissingExternalFile(ConfigMeta),
}

impl ConfigError {
    pub fn to_string(&self) -> String {
        match self {
            &ConfigError::YamlScan(ref e) => format!("Failed to scan YAML: {}", e),
            &ConfigError::YamlParse(ref meta) => {
                let tree = meta.tree();

                let message = if meta.bad_value {
                    "Could not find YAML"
                } else {
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
                } else {
                    "".to_string()
                };

                format!("{}{}", basic, options)
            }
            &ConfigError::NonConfig => "Attempted usage of a struct function on a field.".to_string(),
            &ConfigError::FileError(ref path, ref e) => format!("{}: Config File Error: {}", path.display().to_string(), e),
            &ConfigError::MultipleExternalFiles(ref path, ref conflicts) => {
                let mut result = "".to_string();

                for (index, conflict) in conflicts.iter().enumerate() {
                    if index != 0 {
                        result = result + ",\n\t";
                    }

                    result = result + &conflict.display().to_string();
                }

                format!("{}: Multiple external files: \n\t{}",
                        path.display().to_string(),
                        result)
            }
            &ConfigError::MissingExternalFile(ref meta) => {
                format!("{}: External YAML file is missing",
                        meta.path.display().to_string())
            }
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
    /// Where the file is located, defaults to "config/config.yml"
    pub path: PathBuf,

    /// List from top-level to bottom-level configs
    pub fields: Vec<String>,

    /// Parent meta
    pub parent: Option<Box<ConfigMeta>>,

    /// String representation of the type
    pub ty: &'static str,

    /// Name of the field
    pub name: &'static str,

    /// Whether key is bad or not
    pub bad_value: bool,

    /// Options to display to user, usually used for enums
    pub options: Vec<String>,
}

impl Default for ConfigMeta {
    fn default() -> Self {
        ConfigMeta {
            path: PathBuf::from("config/config.yml"),
            fields: Vec::new(),
            parent: None,
            ty: "Unknown Type",
            name: "Unknown Name",
            bad_value: false,
            options: Vec::new(),
        }
    }
}

impl ConfigMeta {
    /// Displays the meta's fields in order e.g. Config->nested_config->field
    pub fn tree(&self) -> String {
        let mut tree = "".to_string();

        for (index, element) in self.fields.iter().enumerate() {
            if index != 0 {
                tree = tree + "->";
            }

            tree = tree + element;
        }

        tree
    }

    /// Returns the highest level parent or the root meta of the configuration
    pub fn root(&self) -> ConfigMeta {
        if let Some(ref parent) = self.parent {
            parent.root()
        } else {
            self.clone()
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
            _meta: $crate::ConfigMeta,
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )*
        }

        impl $root {
            /// TODO: Needs documentation!
            pub fn to_string(&self) -> String {
                $crate::to_string(&self.to_yaml(&self._meta.path.as_path())) + "\n"
            }
        }

        impl Default for $root {
            fn default() -> Self {
                $root {
                    _meta: $crate::ConfigMeta::default(),
                    $( $field: $name, )*
                }
            }
        }

        impl $crate::Element for $root {
            /// TODO: Needs documentation!
            fn from_yaml(meta: &$crate::ConfigMeta, config: &$crate::Yaml) -> Result<Self, $crate::ConfigError> {
                use std::collections::HashSet;
                use std::path::PathBuf;

                let mut default = $root::default();

                let mut next_meta = meta.clone();
                next_meta.ty = stringify!($root);

                // Appends top-level
                if meta.fields.len() == 0 {
                    next_meta.fields.push(stringify!($root).to_string());
                }

                default._meta = next_meta.clone();

                // Warns of keys that are in the Yaml Hash, but not in the structure
                let mut unexpected = HashSet::new();

                if let &$crate::Yaml::Hash(ref hash) = config {
                    for (key, _) in hash {
                        if let &$crate::Yaml::String(ref key_str) = key {
                            unexpected.insert(key_str.clone());
                        }
                    }

                    $(
                        unexpected.remove(&stringify!($field).to_string());
                    )*

                    for unexpected_key in unexpected {
                        println!("{}: Unexpected key: {}",
                            default._meta.path.display().to_string(),
                            default._meta.tree() + "->" + &unexpected_key);
                    }
                }

                Ok($root {
                    _meta: default._meta,
                    $(
                        $field: {
                            let key = &config[stringify!($field)];

                            // Set up current meta
                            let mut field_meta = next_meta.clone();

                            field_meta.fields.push(stringify!($field).to_string());
                            field_meta.ty = stringify!($ty);
                            field_meta.name = stringify!($field);
                            field_meta.bad_value = key.is_badvalue();
                            field_meta.parent = Some(Box::new(next_meta.clone()));

                            let val = if key.as_str() == Some("extern") { // External file
                                let mut path = PathBuf::from("");

                                for (index, child) in field_meta.fields.iter().enumerate() {
                                    if index != 0 {
                                        path.push(child);
                                    }
                                }

                                <$ty>::from_file_raw(&field_meta, &path.as_path())
                            } else { // current file
                                <$ty>::from_yaml(&field_meta, key)
                            };

                            match val {
                                Ok(found) => found,
                                Err(e) => {
                                    // Output error and fall-through the default values
                                    println!("{}", e);

                                    default.$field.set_meta(&field_meta);

                                    default.$field
                                },
                            }
                        },
                    )*
                })
            }

            /// TODO: Needs documentation!
            fn to_yaml(&self, path: &Path) -> $crate::Yaml {
                use std::collections::BTreeMap;

                let mut map: BTreeMap<$crate::Yaml, $crate::Yaml> = BTreeMap::new();

                $(
                    map.insert(
                        $crate::Yaml::String(stringify!($field).to_string()),
                        self.$field.to_yaml(path),
                    );

                    if let Some(field_meta) = self.$field.meta() {
                        if field_meta.path != path {
                            map.insert(
                                $crate::Yaml::String(stringify!($field).to_string()),
                                $crate::Yaml::String("extern".to_string()),
                            );
                        }
                    }
                )*

                $crate::Yaml::Hash(map)
            }

            /// TODO: Needs documentation!
            fn set_meta(&mut self, meta: &$crate::ConfigMeta) {
                self._meta = meta.clone();
            }

            /// TODO: Needs documentation!
            fn meta(&self) -> Option<$crate::ConfigMeta> {
                Some(self._meta.clone())
            }

            /// TODO: Needs documentation!
            fn write_file(&self) -> Result<(), $crate::ConfigError> {
                use std::fs::{DirBuilder, File};
                use std::io::{Write};

                let path = self._meta.path.clone();
                let readable = self.to_string();

                // Recursively create in the case of new project or deleted folders
                try!(DirBuilder::new().recursive(true).create(&path.parent().unwrap())
                    .map_err(|e| $crate::ConfigError::FileError(path.clone(), e)));

                let mut file = try!(File::create(&path)
                    .map_err(|e| $crate::ConfigError::FileError(path.clone(), e)));
                try!(file.write_all(readable.as_bytes())
                    .map_err(|e| $crate::ConfigError::FileError(path.clone(), e)));

                $(
                    if let Some(ref field_meta) = self.$field.meta() {
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

        impl $crate::Element for $root {
            /// TODO: Needs documentation!
            fn from_yaml(meta: &$crate::ConfigMeta, config: &$crate::Yaml) -> Result<Self, $crate::ConfigError> {
                let mut next_meta = meta.clone();
                next_meta.options = vec![$( stringify!($field).to_string(), )*];

                if let &$crate::Yaml::String(ref string) = config {
                    let s: &str = string;

                    match s {
                        $(
                            stringify!($field) => Ok($root::$field),
                        )*
                        _ => Err($crate::ConfigError::YamlParse(next_meta.clone()))
                    }
                }
                else {
                    Err($crate::ConfigError::YamlParse(next_meta.clone()))
                }
            }

            /// TODO: Needs documentation!
            fn to_yaml(&self, _: &Path) -> $crate::Yaml {
                match self {
                    $(
                        &$root::$field => $crate::Yaml::String(stringify!($field).to_string()),
                    )*
                }
            }
        }
    };
}

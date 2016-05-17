
use std::fs::File;
use std::io::{Read, Error};
use std::path::{PathBuf, Path};
use std::default::Default;
use std::fmt;

use yaml_rust::{Yaml, YamlLoader, ScanError};

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

                format!("{}: Failed to parse YAML: {}: expect {}", meta.path.display(), path, meta.ty)
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
    path: PathBuf, // Where the file is located, "" if not from a file.
    fields: Vec<&'static str>, // List from top-level to bottom-level configs
    ty: &'static str, // String representation of the type
}

impl Default for ConfigMeta {
    fn default() -> Self {
        ConfigMeta {
            path: PathBuf::from(""),
            fields: Vec::new(),
            ty: "T",
        }
    }
}

pub trait FromYaml: Sized {
    /// Convert yaml element into a rust type,
    /// Raises an error if it is not the yaml element expected
    fn from_yaml(&ConfigMeta, &Yaml) -> Result<Self, ConfigError>;
}

macro_rules! yaml_int {
    ($t:ty) => {
        impl FromYaml for $t {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                let num: $t = try!(config.as_i64()
                    .ok_or(ConfigError::YamlParse(meta.clone()))) as $t;
                Ok(num)
            }
        }
    }
}

yaml_int!(i8);
yaml_int!(i16);
yaml_int!(i32);
yaml_int!(i64);
yaml_int!(u8);
yaml_int!(u16);
yaml_int!(u32);
yaml_int!(u64);

impl FromYaml for f32 {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_f64().ok_or(ConfigError::YamlParse(meta.clone()))) as f32)
    }
}

impl FromYaml for f64 {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_f64().ok_or(ConfigError::YamlParse(meta.clone()))))
    }
}

impl FromYaml for bool {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_bool().ok_or(ConfigError::YamlParse(meta.clone()))))
    }
}

impl FromYaml for String {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if let &Yaml::String(ref string) = config {
            Ok(string.clone())
        }
        else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }
}

// Not sure if this is entirely needed
impl FromYaml for () {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(())
        }
        else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }
}

macro_rules! yaml_array {
    ($n:expr => $($i:expr)+) => {
        impl<T: FromYaml> FromYaml for [T; $n] {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                if let &Yaml::Array(ref array) = config {
                    if array.len() != $n {
                        return Err(ConfigError::YamlParse(meta.clone()));
                    }

                    Ok([
                       $(
                            try!(T::from_yaml(meta, &array.get($i).unwrap())
                                .map_err(|_| ConfigError::YamlParse(meta.clone()))),
                       )+
                    ])
                }
                else {
                    Err(ConfigError::YamlParse(meta.clone()))
                }
            }
        }
    }
}

yaml_array!(1 => 0);
yaml_array!(2 => 0 1);
yaml_array!(3 => 0 1 2);
yaml_array!(4 => 0 1 2 3);
yaml_array!(5 => 0 1 2 3 4);
yaml_array!(6 => 0 1 2 3 4 5);
yaml_array!(7 => 0 1 2 3 4 5 6);
yaml_array!(8 => 0 1 2 3 4 5 6 7);
yaml_array!(9 => 0 1 2 3 4 5 6 7 8);
yaml_array!(10 => 0 1 2 3 4 5 6 7 8 9);

impl<T: FromYaml> FromYaml for Option<T> {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(None)
        } else {
            Ok(Some(try!(<T>::from_yaml(meta, config))))
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
                    next_meta.fields.push(stringify!($root));
                }

                default._meta = next_meta.clone();

                Ok($root {
                    _meta: default._meta,
                    $(
                        $field: {
                            let key = &config[stringify!($field)];

                            // set up current meta
                            let mut field_meta = next_meta.clone();
                            field_meta.fields.push(stringify!($field));
                            field_meta.ty = stringify!($ty);

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

// Defines types along with defaulting values
config!(InnerInnerConfig {
    inside: f64 = 5.0,
    other: f32 = 2.5,
});

config!(InnerConfig {
    config: InnerInnerConfig = InnerInnerConfig::default(),
    other_stuff: String = "Hi there".to_string(),
});

config!(DisplayConfig {
    brightness: f64 = 1.0,
    fullscreen: bool = false,
    size: [u16; 2] = [1024, 768],
});

config!(LoggingConfig {
    file_path: String = "new_project.log".to_string(),
    output_level: String = "warn".to_string(),
    logging_level: String = "debug".to_string(),
});

config!(Config {
    test: Option<i64> = Some(58),
    title: String = "Amethyst game".to_string(),
    display: DisplayConfig = DisplayConfig::default(),
    logging: LoggingConfig = LoggingConfig::default(),
    inner: InnerConfig = InnerConfig::default(),
});
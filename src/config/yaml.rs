
use std::fs::File;
use std::io::Read;
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::hash::Hash;
use std::cmp::Eq;
use std::iter;
use std::path::{PathBuf, Path};

use yaml_rust::{Yaml, YamlLoader};

use config::definitions::{ConfigError, ConfigMeta};

pub fn to_string(yaml: &Yaml) -> String {
    to_string_raw(yaml, 0)
}

// Converts a Yaml type into a readable yaml string
fn to_string_raw(yaml: &Yaml, level: usize) -> String {
    match yaml {
        &Yaml::Real(ref value) => value.clone(),
        &Yaml::Integer(ref value) => value.to_string(),
        &Yaml::String(ref value) => value.clone(),
        &Yaml::Boolean(ref value) => value.to_string(),
        &Yaml::Array(ref array) => {
            let mut result = "[".to_string();

            for (index, element) in array.iter().enumerate() {
                //let padding: String = iter::repeat("    ").take(level).collect();

                /*let formatted = format!("\n{}- {}",
                    padding,
                    to_string_raw(element, level + 1)
                );*/

                if index != 0 {
                    result = result + ", ";
                }

                result = result + &to_string_raw(element, level + 1);
            }

            result = result + "]";

            result
        },
        &Yaml::Hash(ref hash) => {
            let mut result = "".to_string();

            for (key, value) in hash {
                let padding: String = iter::repeat("    ").take(level).collect();

                let formatted = format!("\n{}{}: {}",
                    padding,
                    to_string_raw(key, level + 1), to_string_raw(value, level + 1)
                );

                result = result + &formatted;
            }

            result
        },
        &Yaml::Null => "null".to_string(),
        _ => "Bad Value".to_string(), // Should never be a Yaml::BadValue | Yaml::Alias
    }
}

pub trait Element: Sized {
    /// Convert yaml element into a rust type,
    /// Raises an error if it is not the yaml element expected
    fn from_yaml(&ConfigMeta, &Yaml) -> Result<Self, ConfigError>;

    // Converts rust type into a yaml element for writing
    // Requires the path for external configs
    fn to_yaml(&self, &Path) -> Yaml;

    // Only works on structs created by config! macro
    fn set_meta(&mut self, &ConfigMeta) { }

    // Returns meta data if it is a config structure
    fn get_meta(&self) -> Option<ConfigMeta> {
        None
    }

    // From a file relative to current config
    fn from_file_raw(meta: &ConfigMeta, path: &Path) -> Result<Self, ConfigError> {
        let mut next_meta = meta.clone();

        next_meta.path.push(path);

        if next_meta.path.is_dir() && next_meta.path.exists() {
            next_meta.path.push("config");
        }

        next_meta.path.set_extension("yml");

        // extra check for a file that uses the alternate extensions .yaml instead of .yml
        if !next_meta.path.exists() {
            next_meta.path.set_extension("yaml");
        }

        let path = next_meta.path.clone();

        if path.exists() {
            let mut file = try!(File::open(path.as_path())
                .map_err(|e| ConfigError::FileError(path.display().to_string(), e)));
            let mut buffer = String::new();

            try!(file.read_to_string(&mut buffer)
                .map_err(|e| ConfigError::FileError(path.display().to_string(), e)));

            let yaml = try!(YamlLoader::load_from_str(&buffer)
                .map_err(|e| ConfigError::YamlScan(e)));
            let hash = &yaml[0];

            Self::from_yaml(&next_meta, hash)
        }
        else {
            Err(ConfigError::MissingExternalFile(next_meta.clone()))
        }
    }

    // From a file relative to project
    fn from_file(path: &Path) -> Result<Self, ConfigError> {
        Self::from_file_raw(&ConfigMeta::default(), path)
    }

    fn write_file(&self) -> Result<(), ConfigError> {
        Err(ConfigError::YamlGeneric("Attempting to write on a non-config struct".to_string()))
    }
}

macro_rules! yaml_int {
    ($t:ty) => {
        impl Element for $t {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                let num: $t = try!(config.as_i64()
                    .ok_or(ConfigError::YamlParse(meta.clone()))) as $t;
                Ok(num)
            }

            fn to_yaml(&self, _: &Path) -> Yaml {
                Yaml::Integer(self.clone() as i64)
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

impl Element for f32 {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_f64().ok_or(ConfigError::YamlParse(meta.clone()))) as f32)
    }

    fn to_yaml(&self, _: &Path) -> Yaml {
        Yaml::Real(self.clone().to_string())
    }
}

impl Element for f64 {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_f64().ok_or(ConfigError::YamlParse(meta.clone()))))
    }

    fn to_yaml(&self, _: &Path) -> Yaml {
        Yaml::Real(self.clone().to_string())
    }
}

impl Element for bool {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        Ok(try!(config.as_bool().ok_or(ConfigError::YamlParse(meta.clone()))))
    }

    fn to_yaml(&self, _: &Path) -> Yaml {
        Yaml::Boolean(self.clone())
    }
}

impl Element for String {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if let &Yaml::String(ref string) = config {
            Ok(string.clone())
        }
        else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }

    fn to_yaml(&self, _: &Path) -> Yaml {
        Yaml::String(self.clone())
    }
}

// Not sure if this is entirely needed
impl Element for () {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(())
        }
        else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }

    fn to_yaml(&self, _: &Path) -> Yaml {
        Yaml::Null
    }
}

impl<T: Element> Element for Option<T> {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(None)
        }
        else {
            Ok(Some(try!(<T>::from_yaml(meta, config))))
        }
    }

    fn to_yaml(&self, path: &Path) -> Yaml {
        match self {
            &Some(ref val) => val.to_yaml(path),
            &None => Yaml::Null,
        }
    }
}

macro_rules! yaml_array {
    ($n:expr => $($i:expr)+) => {
        impl<T: Element> Element for [T; $n] {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                if let &Yaml::Array(ref array) = config {
                    if array.len() != $n {
                        return Err(ConfigError::YamlParse(meta.clone()));
                    }

                    Ok([
                       $(
                            try!(<T>::from_yaml(meta, &array.get($i).unwrap())
                                .map_err(|_| ConfigError::YamlParse(meta.clone()))),
                       )+
                    ])
                }
                else {
                    Err(ConfigError::YamlParse(meta.clone()))
                }
            }

            fn to_yaml(&self, path: &Path) -> Yaml {
                let mut vec: Vec<Yaml> = Vec::new();

                for element in self {
                    vec.push(element.to_yaml(path));
                }

                Yaml::Array(vec)
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

impl<T: Element> Element for Vec<T> {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if let &Yaml::Array(ref array) = config {
            let mut vec = Vec::new();

            for (index, element) in array.iter().enumerate() {
                let mut element_meta = meta.clone();
                element_meta.fields.push(index.to_string());

                vec.push(try!(<T>::from_yaml(&element_meta, element)));
            }

            Ok(vec)
        }
        else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }

    fn to_yaml(&self, path: &Path) -> Yaml {
        let mut vec: Vec<Yaml> = Vec::new();

        for element in self {
            vec.push(element.to_yaml(path));
        }

        Yaml::Array(vec)
    }
}

macro_rules! yaml_map {
    ( $map:ident: $( $bound:ident )* ) => {
        impl<K: Element $( + $bound )*, V: Element> Element for $map<K, V> {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                if let &Yaml::Hash(ref hash) = config {
                    let mut map = $map::new();

                    for (key, value) in hash.iter() {
                        let mut key_meta = meta.clone();
                        key_meta.fields.push("key".to_string());

                        let mut value_meta = meta.clone();
                        value_meta.fields.push("value".to_string());

                        map.insert(
                            try!(<K>::from_yaml(&key_meta, key)),
                            try!(<V>::from_yaml(&value_meta, value))
                        );
                    }

                    Ok(map)
                }
                else {
                    Err(ConfigError::YamlParse(meta.clone()))
                }
            }

            fn to_yaml(&self, path: &Path) -> Yaml {
                use std::collections::BTreeMap;

                let mut map: BTreeMap<Yaml, Yaml> = BTreeMap::new();

                for (key, value) in self.iter() {
                    map.insert(
                        key.to_yaml(path),
                        value.to_yaml(path),
                    );
                }

                Yaml::Hash(map)
            }
        }
    }
}

yaml_map!(HashMap: Hash Eq);
yaml_map!(BTreeMap: Ord);

macro_rules! yaml_set {
    ( $set:ident: $( $bound:ident )* ) => {
        impl<T: Element $( + $bound )*> Element for $set<T> {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                if let &Yaml::Array(ref list) = config {
                    let mut set = $set::new();

                    for element in list.iter() {
                        set.insert(
                            try!(<T>::from_yaml(&meta, element))
                        );
                    }

                    Ok(set)
                }
                else {
                    Err(ConfigError::YamlParse(meta.clone()))
                }
            }

            fn to_yaml(&self, path: &Path) -> Yaml {
                let mut vec: Vec<Yaml> = Vec::new();

                for element in self.iter() {
                    vec.push(
                        element.to_yaml(path),
                    );
                }

                Yaml::Array(vec)
            }
        }
    }
}

yaml_set!(HashSet: Hash Eq);
yaml_set!(BTreeSet: Ord);
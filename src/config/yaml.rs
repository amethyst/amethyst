
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::hash::Hash;
use std::cmp::Eq;

use yaml_rust::{Yaml, YamlLoader, ScanError};

use config::definitions::{ConfigError, ConfigMeta};

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

impl<T: FromYaml> FromYaml for Option<T> {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(None)
        } else {
            Ok(Some(try!(<T>::from_yaml(meta, config))))
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

impl<T: FromYaml> FromYaml for Vec<T> {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if let &Yaml::Array(ref array) = config {
            let mut vec = Vec::new();

            for (index, element) in array.iter().enumerate() {
                let mut element_meta = meta.clone();
                element_meta.fields.push(index.to_string());

                vec.push(try!(<T>::from_yaml(&element_meta, element)));
            }

            Ok(vec)
        } else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }
}

macro_rules! yaml_map {
    ( $map:ident: $( $bound:ident )* ) => {
        impl<K: FromYaml $( + $bound )*, V: FromYaml> FromYaml for $map<K, V> {
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
                } else {
                    Err(ConfigError::YamlParse(meta.clone()))
                }
            }
        }
    }
}

yaml_map!(HashMap: Hash Eq);
yaml_map!(BTreeMap: Ord);

macro_rules! yaml_set {
    ( $set:ident: $( $bound:ident )* ) => {
        impl<T: FromYaml $( + $bound )*> FromYaml for $set<T> {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                if let &Yaml::Array(ref list) = config {
                    let mut set = $set::new();

                    for element in list.iter() {
                        set.insert(
                            try!(<T>::from_yaml(&meta, element))
                        );
                    }

                    Ok(set)
                } else {
                    Err(ConfigError::YamlParse(meta.clone()))
                }
            }
        }
    }
}

yaml_set!(HashSet: Hash Eq);
yaml_set!(BTreeSet: Ord);
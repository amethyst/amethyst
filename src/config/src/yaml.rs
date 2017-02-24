
//! Configuration structure elements and conversions

use std::fs::File;
use std::io::Read;
use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::hash::Hash;
use std::cmp::Eq;
use std::iter;
use std::path::{PathBuf, Path};

use yaml_rust::{Yaml, YamlLoader};

use definitions::{ConfigError, ConfigMeta};

static TAB_CHARS: &'static str = "    "; // Characters to display for tabs

/// Converts a Yaml object into a .yml/.yaml format
pub fn to_string(yaml: &Yaml) -> String {
    to_string_raw(yaml, 0)
}

/// Converts a Yaml type into a readable yaml string
fn to_string_raw(yaml: &Yaml, level: usize) -> String {
    match yaml {
        &Yaml::Real(ref value) => {
            let mut float_string = value.to_string();

            // Rust automatically truncates floats without a fractional in them
            // So stuff like "3.0" becomes "3" and the parser can't read it.
            if float_string.find(".") == None {
                float_string = float_string + ".0";
            }

            float_string
        }
        &Yaml::Integer(ref value) => value.to_string(),
        &Yaml::String(ref value) => value.clone(),
        &Yaml::Boolean(ref value) => value.to_string(),
        &Yaml::Array(ref array) => {
            // Check if the array has an array, hash, or is long to determine display type
            let mut complex = false;
            for element in array {
                match element {
                    &Yaml::Array(_) | &Yaml::Hash(_) => {
                        complex = true;
                    }
                    _ => {}
                }
            }

            if array.len() > 10 {
                complex = true;
            }

            let mut result = "".to_string();

            for (index, element) in array.iter().enumerate() {
                if index != 0 && !complex {
                    result = result + ", ";
                }

                if complex {
                    let padding: String = iter::repeat(TAB_CHARS).take(level).collect();
                    let formatted = format!("\n{}- {}", padding, to_string_raw(element, level + 2));
                    result = result + &formatted;
                } else {
                    result = result + &to_string_raw(element, level + 1);
                }
            }

            if !complex {
                result = "[".to_string() + &result + "]";
            }

            result
        }
        &Yaml::Hash(ref hash) => {
            let mut result = "".to_string();

            for (key, value) in hash {
                let padding: String = iter::repeat(TAB_CHARS).take(level).collect();
                let formatted = format!("\n{}{}: {}",
                                        padding,
                                        to_string_raw(key, level + 1),
                                        to_string_raw(value, level + 1));

                result = result + &formatted;
            }

            result
        }
        &Yaml::Null => "null".to_string(),
        _ => "Bad Value".to_string(), // Should never be a Yaml::BadValue | Yaml::Alias
    }
}

/// Trait for fields inside of a configuration struct.
pub trait Element: Sized {
    /// Convert yaml element into a rust type,
    /// Raises an error if it is not the yaml element expected
    fn from_yaml(&ConfigMeta, &Yaml) -> Result<Self, ConfigError>;

    /// Converts rust type into a yaml element for writing
    /// Requires the path for external configs
    fn to_yaml(&self, &Path) -> Yaml;

    /// Sets the meta data of a config structure, only works on config structures
    fn set_meta(&mut self, &ConfigMeta) {}

    /// Returns meta data if it is a config structure
    fn meta(&self) -> Option<ConfigMeta> {
        None
    }

    /// From some string (should be used for top level elements if you want to embed the code)
    fn from_string(src: &str) -> Result<Self, ConfigError> {
        let mut meta = ConfigMeta::default();
        meta.path = PathBuf::from("");
        let yaml = YamlLoader::load_from_str(src).map_err(|e| ConfigError::YamlScan(e))?;

        let hash = if yaml.len() > 0 {
            yaml[0].clone()
        } else {
            Yaml::Hash(BTreeMap::new())
        };

        Self::from_yaml(&meta, &hash)
    }

    /// From a file relative to current config
    fn from_file_raw<P: AsRef<Path>>(meta: &ConfigMeta, path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut next_meta = meta.clone();

        let initial_path = if next_meta.path.is_file() {
            if let Some(parent) = next_meta.path.parent() {
                parent.to_path_buf()
            } else {
                PathBuf::from("")
            }
        } else {
            next_meta.path.clone()
        };

        let check = |list: &mut Vec<PathBuf>, file: &mut PathBuf| if file.exists() {
            list.push(file.clone())
        };

        let mut found = Vec::new();

        // file .yml
        let mut file_path = initial_path.clone();
        file_path.push(path);
        file_path.set_extension("yml");

        // for proper error messages, displays the path it is looking for instead of parent
        next_meta.path = file_path.clone();

        check(&mut found, &mut file_path);

        // file .yaml
        file_path.set_extension("yaml");
        check(&mut found, &mut file_path);

        // dir .yml
        file_path.set_extension("");
        file_path.push("config");
        file_path.set_extension("yml");
        check(&mut found, &mut file_path);

        // dir .yaml
        file_path.set_extension("yaml");
        check(&mut found, &mut file_path);

        if found.len() > 1 {
            return Err(ConfigError::MultipleExternalFiles(path.to_path_buf(), found));
        } else if found.len() == 0 {
            return Err(ConfigError::MissingExternalFile(next_meta.clone()));
        }

        let found_path = found[0].clone();
        next_meta.path = found_path.clone();

        let mut file = File::open(found_path.as_path()).map_err(|e| ConfigError::FileError(found_path.clone(), e))?;
        let mut buffer = String::new();

        file.read_to_string(&mut buffer)
            .map_err(|e| ConfigError::FileError(found_path.clone(), e))?;

        let yaml = YamlLoader::load_from_str(&buffer).map_err(|e| ConfigError::YamlScan(e))?;

        let hash = if yaml.len() > 0 {
            yaml[0].clone()
        } else {
            Yaml::Hash(BTreeMap::new())
        };

        Self::from_yaml(&next_meta, &hash)
    }

    /// From a file relative to project
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let mut default = ConfigMeta::default();
        default.path = PathBuf::from("");

        Self::from_file_raw(&default, path)
    }

    /// Recursively writes to files given the configuration's current context.
    ///
    /// The default path for a root configuration file is "config/config.yml".
    ///
    /// Note: This should never be called on a non-config! defined structure.
    fn write_file(&self) -> Result<(), ConfigError> {
        Err(ConfigError::NonConfig)
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
        } else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }

    fn to_yaml(&self, _: &Path) -> Yaml {
        Yaml::String("\"".to_string() + &self.clone() + "\"")
    }
}

impl Element for () {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(())
        } else {
            Err(ConfigError::YamlParse(meta.clone()))
        }
    }

    fn to_yaml(&self, _: &Path) -> Yaml {
        Yaml::Null
    }
}

macro_rules! yaml_tuple {
    ($length:expr => $($name:ident : $position:expr,)+) => {
        impl< $($name: Element),* > Element for ( $($name,)* ) {
            fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
                if let &Yaml::Array(ref array) = config {
                    if array.len() < $length {
                        return Err(ConfigError::YamlParse(meta.clone()));
                    }

                    Ok((
                        $( try!(<$name>::from_yaml(&meta.clone(), &array[$position])), )*
                    ))
                }
                else {
                    Err(ConfigError::YamlParse(meta.clone()))
                }
            }

            fn to_yaml(&self, path: &Path) -> Yaml {
                #![allow(non_snake_case)]
                let mut arr = Vec::new();
                let &($(ref $name,)*) = self;

                $(
                    arr.push($name.to_yaml(path));
                )*

                Yaml::Array(arr)
            }
        }
    }
}

yaml_tuple!(1 => A:0,);
yaml_tuple!(2 => A:0, B:1,);
yaml_tuple!(3 => A:0, B:1, C:2,);
yaml_tuple!(4 => A:0, B:1, C:2, D:3,);
yaml_tuple!(5 => A:0, B:1, C:2, D:3, E:4,);
yaml_tuple!(6 => A:0, B:1, C:2, D:3, E:4, F:5,);
yaml_tuple!(7 => A:0, B:1, C:2, D:3, E:4, F:5, G:6,);
yaml_tuple!(8 => A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7,);
yaml_tuple!(9 => A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8,);
yaml_tuple!(10 => A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9,);

impl<T: Element> Element for Option<T> {
    fn from_yaml(meta: &ConfigMeta, config: &Yaml) -> Result<Self, ConfigError> {
        if config.is_null() {
            Ok(None)
        } else {
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
        } else {
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

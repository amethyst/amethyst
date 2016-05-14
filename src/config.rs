
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::fmt;

use yaml_rust::{Yaml, YamlLoader};


pub trait FromYaml: Sized {
    /// Convert yaml element into a rust type,
    /// Raises an error if it is not the yaml element expected
    fn from_yaml(Yaml) -> Result<Self, String>;
}

impl FromYaml for i64 {
    fn from_yaml(config: Yaml) -> Result<Self, String> {
        Ok(try!(config.as_i64().ok_or("expect integer")))
    }
}

impl FromYaml for f64 {
    fn from_yaml(config: Yaml) -> Result<Self, String> {
        Ok(try!(config.as_f64().ok_or("expect float")))
    }
}

impl FromYaml for bool {
    fn from_yaml(config: Yaml) -> Result<Self, String> {
        Ok(try!(config.as_bool().ok_or("expect boolean")))
    }
}

impl FromYaml for String {
    fn from_yaml(config: Yaml) -> Result<Self, String> {
        if let Yaml::String(string) = config {
            Ok(string)
        } else {
            Err("expect string".into())
        }
    }
}

impl FromYaml for () {
    fn from_yaml(config: Yaml) -> Result<Self, String> {
        if config.is_null() {
            Ok(())
        } else {
            Err("expect null".into())
        }
    }
}

macro_rules! yaml_array {
    ($t:ty => $n:expr => $($i:expr)+) => {
        impl FromYaml for [$t;$n] {
            fn from_yaml(config: Yaml) -> Result<Self, String> {
                if let Yaml::Array(mut array) = config {
                    if array.len() != $n { return Err(format!("expect list of length {}",$n).into()); }

                    Ok([
                       $(
                           try!(<$t>::from_yaml(array.remove(0))
                                .map_err(|e| format!("list[{}]: {}",$i,e))),
                       )+
                    ])
                } else {
                    Err("expect list".into())
                }
            }
        }
    }
}

// This is pretty horrific, a generic trait over FromYaml doesn't seem to work though.
// Any ideas?
yaml_array!(i64 => 1 => 0);
yaml_array!(i64 => 2 => 0 1);
yaml_array!(i64 => 3 => 0 1 2);
yaml_array!(i64 => 4 => 0 1 2 3);
yaml_array!(i64 => 5 => 0 1 2 3 4);
yaml_array!(i64 => 6 => 0 1 2 3 4 5);
yaml_array!(i64 => 7 => 0 1 2 3 4 5 6);
yaml_array!(i64 => 8 => 0 1 2 3 4 5 6 7);
yaml_array!(i64 => 9 => 0 1 2 3 4 5 6 7 8);
yaml_array!(i64 => 10 => 0 1 2 3 4 5 6 7 8 9);

yaml_array!(f64 => 1 => 0);
yaml_array!(f64 => 2 => 0 1);
yaml_array!(f64 => 3 => 0 1 2);
yaml_array!(f64 => 4 => 0 1 2 3);
yaml_array!(f64 => 5 => 0 1 2 3 4);
yaml_array!(f64 => 6 => 0 1 2 3 4 5);
yaml_array!(f64 => 7 => 0 1 2 3 4 5 6);
yaml_array!(f64 => 8 => 0 1 2 3 4 5 6 7);
yaml_array!(f64 => 9 => 0 1 2 3 4 5 6 7 8);
yaml_array!(f64 => 10 => 0 1 2 3 4 5 6 7 8 9);

yaml_array!(bool => 1 => 0);
yaml_array!(bool => 2 => 0 1);
yaml_array!(bool => 3 => 0 1 2);
yaml_array!(bool => 4 => 0 1 2 3);
yaml_array!(bool => 5 => 0 1 2 3 4);
yaml_array!(bool => 6 => 0 1 2 3 4 5);
yaml_array!(bool => 7 => 0 1 2 3 4 5 6);
yaml_array!(bool => 8 => 0 1 2 3 4 5 6 7);
yaml_array!(bool => 9 => 0 1 2 3 4 5 6 7 8);
yaml_array!(bool => 10 => 0 1 2 3 4 5 6 7 8 9);

yaml_array!(String => 1 => 0);
yaml_array!(String => 2 => 0 1);
yaml_array!(String => 3 => 0 1 2);
yaml_array!(String => 4 => 0 1 2 3);
yaml_array!(String => 5 => 0 1 2 3 4);
yaml_array!(String => 6 => 0 1 2 3 4 5);
yaml_array!(String => 7 => 0 1 2 3 4 5 6);
yaml_array!(String => 8 => 0 1 2 3 4 5 6 7);
yaml_array!(String => 9 => 0 1 2 3 4 5 6 7 8);
yaml_array!(String => 10 => 0 1 2 3 4 5 6 7 8 9);

macro_rules! config {
    ($root:ident {
        $( $conf:ident: $conf_type:ident {
            $( $field:ident: $field_type:ty = $name:expr, )*
        }, )*
    }) => {
        $(
            #[derive(Clone)]
            pub struct $conf_type {
                $( pub $field: $field_type, )*
            }
            
            impl $conf_type {
                pub fn default() -> $conf_type {

                    $conf_type {
                        $( $field: $name, )*
                    }
                }

                pub fn from_yaml(hash: &Yaml) -> $conf_type {
                    let mut default = $conf_type::default();

                    let conf_name = stringify!($conf);

                    $(
                        let field_name = stringify!($field);

                        if !&hash[conf_name][field_name].is_badvalue() {
                            match <$field_type>::from_yaml(hash[conf_name][field_name].clone()) {
                                Ok(value) => {
                                    default.$field = value;
                                },
                                Err(e) => {
                                    println!("Config mismatch: {:?}.{:?}: {:?}", conf_name, field_name, e);
                                },
                            }
                        }
                    )*

                    default
                }
            }

            // Easier to read debug
            impl fmt::Debug for $conf_type {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    let mut fields = "".to_owned();
                    $( 
                        fields = fields + "\t\t" + &format!(r"{}: {:?}", stringify!($field), self.$field) + "\n\r";
                    )*
                    write!(f, "{} {{\n {} \t}}", stringify!($conf), fields)
                }
            }
        )*
    
        #[derive(Clone)]
        pub struct $root {
            $( pub $conf: $conf_type, )*
        }
        
        impl $root {
            pub fn default() -> $root {
                $root {
                    $( $conf: $conf_type::default(), )*
                }
            }

            pub fn from_file(path: &Path) -> $root {
                let mut file = File::open(path).unwrap();
                let mut buffer = String::new();
                file.read_to_string(&mut buffer);

                let yaml = YamlLoader::load_from_str(&buffer).unwrap();
                let hash = &yaml[0];

                $root {
                    $(
                        $conf: $conf_type::from_yaml(&hash),
                    )*
                }
            }
        } 

        impl fmt::Debug for $root {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut fields = "".to_owned();
                $( 
                    fields = fields + "\t" + &format!(r"{:?},", self.$conf) + "\n\r";
                )*
                write!(f, "{} {{\n {} }}", stringify!($root), fields)
            }
        }
    }
}

// Defines types along with defaulting values
config!(
    Config {
        display: DisplayConfig {
            brightness: f64 = 1.0,
            fullscreen: bool = false,
            size: [i64; 2] = [1024, 768],
        },
        logging: LoggingConfig {
            file_path: String = "new_project.log".to_string(),
            output_level: String = "warn".to_string(),
            logging_level: String = "debug".to_string(),
        },
    }
);

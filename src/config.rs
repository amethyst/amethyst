
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::fmt;

use yaml_rust::{Yaml, YamlLoader};


pub trait FromYaml: Sized {
    /// Convert yaml element into a rust type,
    /// Raises an error if it is not the yaml element expected
    fn from_yaml(&Yaml) -> Result<Self, String>;
}

macro_rules! yaml_int {
    ($t:ty) => {
        impl FromYaml for $t {
            fn from_yaml(config: &Yaml) -> Result<Self, String> {
                let num: $t = try!(config.as_i64().ok_or("expect integer")) as $t;
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
    fn from_yaml(config: &Yaml) -> Result<Self, String> {
        Ok(try!(config.as_f64().ok_or("expect float")) as f32)
    }
}

impl FromYaml for f64 {
    fn from_yaml(config: &Yaml) -> Result<Self, String> {
        Ok(try!(config.as_f64().ok_or("expect float")))
    }
}

impl FromYaml for bool {
    fn from_yaml(config: &Yaml) -> Result<Self, String> {
        Ok(try!(config.as_bool().ok_or("expect boolean")))
    }
}

impl FromYaml for String {
    fn from_yaml(config: &Yaml) -> Result<Self, String> {
        if let &Yaml::String(ref string) = config {
            Ok(string.clone())
        } else {
            Err("expect string".into())
        }
    }
}

impl FromYaml for () {
    fn from_yaml(config: &Yaml) -> Result<Self, String> {
        if config.is_null() {
            Ok(())
        } else {
            Err("expect null".into())
        }
    }
}

macro_rules! yaml_array {
    ($n:expr => $($i:expr)+) => {
        impl<T: FromYaml> FromYaml for [T;$n] {
            fn from_yaml(config: &Yaml) -> Result<Self,String> {
                if let &Yaml::Array(ref array) = config {
                    if array.len() != $n { return Err(format!("expect list of length {}",$n).into()); }

                    let mut arr = array.clone();

                    Ok([
                       $(
                           try!(T::from_yaml(&arr.remove(0))
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

/*macro_rules! config {
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
            size: [u16; 2] = [1024, 768],
        },
        logging: LoggingConfig {
            file_path: String = "new_project.log".to_string(),
            output_level: String = "warn".to_string(),
            logging_level: String = "debug".to_string(),
        },
    }
);
*/

macro_rules! config {
    ($root:ident {
        $( $field:ident: $ty:ty = $name:expr, )*
    }) => {

        #[derive(Clone)]
        pub struct $root {
            $( pub $field: $ty, )*
        }

        impl $root {
            pub fn default() -> $root {
                $root {
                    $( $field: $name, )*
                }
            }

            pub fn from_file(path: &Path) -> $root {
                let mut file = File::open(path).unwrap();
                let mut buffer = String::new();
                file.read_to_string(&mut buffer);

                let yaml = YamlLoader::load_from_str(&buffer).unwrap();
                let hash = &yaml[0];

                $root::from_yaml(hash).unwrap()
            }
        }

        impl FromYaml for $root {
            fn from_yaml(config: &Yaml) -> Result<Self, String> {
                Ok($root {
                    $(
                        $field: try!(<$ty>::from_yaml(&config[stringify!($field)])),
                    )*
                })
            }
        }

        impl fmt::Debug for $root {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut fields = "".to_owned();
                $(
                    fields = fields + &format!(r"{}: {:?},", stringify!($field), self.$field) + "\n\r";
                )*
                write!(f, "{} {{\n{}}}", stringify!($root), fields)
            }
        }
    }
}

// Defines types along with defaulting values
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

config!(
    Config {
        display: DisplayConfig = DisplayConfig::default(),
        logging: LoggingConfig = LoggingConfig::default(),
    }
);
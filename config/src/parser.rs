use build;
use serde_yaml;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use failure::Error;

pub const DEFAULT_MODULE_TYPE: &'static str = "conan";
pub const DEFAULT_BUILD_FOLDER: &'static str = "build";

#[derive(Deserialize, Debug)]
struct Config {
    default: Option<Default>,
    module: Vec<Module>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Default {
    #[serde(rename="type")]
    module_type: Option<String>,
    config: Option<BTreeMap<String, Value>>
}

#[derive(Deserialize, Debug)]
struct Module {
    name: String,
    path: Option<String>,
    module_type: Option<String>,
    deps: Option<Vec<String>>,
    config: Option<BTreeMap<String, Value>>
}

impl Default {
    fn new() -> Default {
        Default {
            module_type: None,
            config: None
        }
    }
}

pub fn parse_file(context: &mut build::Context, f: &Path) -> Result<(), Error> {
    let mut file = File::open(f)?;
    let mut filecontent : String = String::new();
    file.read_to_string(&mut filecontent)?;
    return parse(context, &filecontent);
}

pub fn parse(context: &mut build::Context, s: &String) -> Result<(), Error> {
    let mut build: Config = serde_yaml::from_str(s)?;

    debug!("{:#?}", build);

    if build.default == None {
        build.default = Some(Default::new())
    }

    // Set a default plugin.
    if build.default.as_ref().unwrap().module_type == None {
        build.default.as_mut().unwrap().module_type = Some(DEFAULT_MODULE_TYPE.to_string());
    }

    // Add modules
    for m in &build.module {
        let mut config : HashMap<String, String> = HashMap::new();
        let plugin = m.module_type.as_ref().unwrap_or(build.default.as_ref().unwrap().module_type.as_ref().unwrap());

        let default_config = build.default.as_ref().unwrap();

        if let Some(default_config) = &default_config.config {
            add_config_to_map(&mut config, &default_config);
        }

        if let Some(module_conf) = &m.config {
            add_config_to_map(&mut config, &module_conf)
        }

        let module_path = match &m.path {
            Some(p) => PathBuf::from(p),
            None => PathBuf::from(&m.name)
        };
        let build_dir = module_path.join(DEFAULT_BUILD_FOLDER);
        let mut types = HashSet::new();
        types.insert(plugin.clone());

        context.add_module(
            &m.name,
            build::Module::new(
                &m.name,
                module_path,
                build_dir,
                types,
                config
            )
        );
    }

    // Add module dependencies
    for m in &build.module {
        if let Some(deps) = &m.deps {
            for dep in deps {
                context.module(&m.name)?.depends_on(dep)?;
            }
        }
    }

    return Ok(());
}

fn add_config_to_map(config: &mut HashMap<String,String>, src: &BTreeMap<String,Value>) {
    for (key,v) in src {
        match v {
            Value::String(v) => {config.insert(key.clone(), v.clone());},
            Value::Bool(v) => {config.insert(key.clone(), v.to_string());},
            Value::Number(v) => {config.insert(key.clone(), v.to_string());},
            Value::Sequence(s) => {
                // TODO: Handle these "unexpected" values more gracefully.
                let value : Vec<String> = s.iter()
                    .map(|x| {
                        match x {
                            Value::String(x) => x.clone(),
                            Value::Bool(b) => b.to_string(),
                            Value::Mapping(_) => "unexpected_map".to_string(),
                            Value::Null => "null".to_string(),
                            Value::Number(n) => n.to_string(),
                            Value::Sequence(_) => "unexpected_sequence".to_string()
                        }
                    }).collect();
                config.insert(key.clone(), value.join(","));
            },
            v => panic!(format!("Config value for key {} must be a string but was {:?}.", key.clone(), v))
        }
    }
}
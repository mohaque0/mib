use ::Plugin;
use failure::Error;
use ::conan::ConanPlugin;
use std::fs;
use std::path::PathBuf;

pub struct Framework {
    plugin_dir: PathBuf
}

impl Framework {
    pub fn new(plugin_dir: &PathBuf) -> Framework {
        Framework {
            plugin_dir: plugin_dir.clone()
        }
    }

    pub fn get_plugins(&self) -> Result<Vec<Box<Plugin>>, Vec<Error>> {
        let mut ret_val : Vec<Box<Plugin>> = vec!(Box::new(ConanPlugin::new()));
    
        /* TODO: Implement ExecutablePlugin
        let plugins = match fs::read_dir(&self.plugin_dir) {
            Ok(iter) => iter,
            Err(e) => {
                ret_val.push(Err(Error::IOError(e.to_string())));
                return Ok(ret_val);
            }
        };

        for plugin in plugins {
            match plugin {
                Ok(p) => ret_val.push(Ok(Box::new(ExecutablePlugin::new(&p.path())?))),
                Err(e) => ret_val.push(Err(Error::IOError(e.to_string())))
            };
        }
        */

        Ok(ret_val)
    }
}
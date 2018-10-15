use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Module {
    name: String,
    module_dir: PathBuf,
    build_dir: PathBuf,
    types: HashSet<String>,
    config: HashMap<String, String>
}

impl Module {
    pub fn new(
        name: &str,
        module_dir: PathBuf,
        build_dir: PathBuf,
        types: HashSet<String>,
        config: HashMap<String, String>
    ) -> Module {
        Module {
            name: name.to_string(),
            module_dir: module_dir,
            build_dir: build_dir,
            types: types,
            config: config
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn module_dir(&self) -> &PathBuf {
        &self.module_dir
    }

    pub fn build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    pub fn types(&self) -> &HashSet<String> {
        &self.types
    }

    pub fn config(&self) -> &HashMap<String, String> {
        &self.config
    }
}
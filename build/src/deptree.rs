use failure::Error;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct DepTree<T: ?Sized> {
    deps: HashMap<String, HashSet<String>>,
    items: HashMap<String, Box<T>>
}

#[derive(Debug, Fail)]
enum DepTreeError {
    #[fail(display = "dependency not found: {}", name)]
    ItemNotFound { name: String }
}

impl<T: ?Sized> DepTree<T> {
    pub fn new() -> DepTree<T> {
        DepTree::<T> {
            deps: HashMap::new(),
            items: HashMap::new()
        }
    }

    pub fn insert_box(&mut self, name: &str, value: Box<T>) {
        self.items.insert(String::from(name), value);
        self.deps.insert(String::from(name), HashSet::new());
    }

    pub fn get_item(&self, name: &str) -> Option<&T> {
        match self.items.get(name) {
            Some(a) => Some(a.as_ref()),
            None => None
        }
    }

    pub fn get_deps(&self, name: &str) -> Result<&HashSet<String>, Error> {
        Ok(self.deps.get(&name.to_string()).ok_or(DepTreeError::ItemNotFound { name: name.to_string() })?)
    }

    pub fn set_dependency(&mut self, dependent: &str, depended_on: &str) -> Result<(), Error> {
        if !self.items.contains_key(dependent) {
            return Err(DepTreeError::ItemNotFound { name: dependent.to_string() })?
        }
        if !self.items.contains_key(depended_on) {
            return Err(DepTreeError::ItemNotFound { name: depended_on.to_string() })?
        }

        self.deps.get_mut(&dependent.to_string()).unwrap().insert(depended_on.to_string());
        Ok(())
    }

    pub fn items_mut(&mut self) -> &HashMap<String, Box<T>> {
        &self.items
    }
}

impl<T: Sized> DepTree<T> {
    pub fn insert(&mut self, name: &str, value: T) {
        self.items.insert(String::from(name), Box::new(value));
        self.deps.insert(String::from(name), HashSet::new());
    }
}
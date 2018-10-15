use build::Context;
use failure::Error;
use std::collections::HashMap;

pub trait Plugin {
    fn get_documentation(&self) -> HashMap<String, String>;
    fn configure(&self, context: &mut Context) -> Result<(), Error>;
}
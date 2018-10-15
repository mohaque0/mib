extern crate build;
extern crate serde_yaml;
extern crate simple_logger;
extern crate yaml_rust;

#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

mod parser;

pub use parser::parse;
pub use parser::parse_file;
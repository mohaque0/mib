use failure::Error;

pub trait Task {
    fn needs_execution(&self) -> bool;
    fn execute(&self) -> Result<(), Error>;
}
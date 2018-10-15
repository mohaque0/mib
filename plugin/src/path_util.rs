use std::convert::From;
use std::path::Path;
use std::path::PathBuf;

pub struct PathBuilder {
    path: PathBuf
}

impl PathBuilder {
    pub fn push<P: AsRef<Path>>(&mut self, path: P) -> &mut PathBuilder {
        self.path.push(path);
        self
    }

    pub fn build(&mut self) -> PathBuf {
        self.into()
    }
}

impl <'a> From<&'a PathBuf> for PathBuilder {
    fn from(path: &PathBuf) -> Self {
        PathBuilder {
            path: path.clone()
        }
    }
}

impl <'a> From<&'a mut PathBuilder> for PathBuf {
    fn from(builder: &mut PathBuilder) -> Self {
        builder.path.clone()
    }
}
use std::path::PathBuf;
use crate::init::Args;

pub struct Application {
    exec_file_path: PathBuf,
    args: Args
}
impl Application {
    pub fn new(exec_path: PathBuf, args: Args) -> Self {
        Application { exec_file_path: exec_path, args: args }
    }
}


use crate::Args;
use crate::core::Solution;
use crate::data::Data;
use regex::Regex;
use std::path::Path;
use std::path::PathBuf;
pub struct Application {
    pub patterns: Vec<Regex>,
    pub exec_path: PathBuf,
    pub args: Args,
}
pub struct State {
    pub data: Data,
    pub args: Args,
}
impl Application {
    pub fn new(patterns: Vec<Regex>, exec_path: PathBuf, args: Args) -> Self {
        if patterns.len() == 0 {
            panic!("Fatal error. No pattern in `patterns`.")
        }
        Application {
            patterns: patterns,
            exec_path: exec_path,
            args: args,
        }
    }
    pub fn start(&self) {
        let data_path: String = format!(
            "{}/data/data.bin",
            self.exec_path.parent().unwrap().display()
        );
        if !self.patterns[0].is_match(&self.args.anagram) {
            panic!("Fatal Error. Invalid characters. Please provide a valid anagram.");
        }
        let data: Data = match Data::try_from(Path::new(&data_path)) {
            Ok(data) => data,
            Err(err) => panic!(
                "Fatal error. Cannot retrieve data at {data_path}: {:?}",
                err
            ),
        };
        let solution: Solution = Solution::from(State {
            data: data,
            args: self.args.clone(),
        });
    }
}

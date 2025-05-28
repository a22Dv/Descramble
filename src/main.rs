use application::Application;
use std::env::current_exe;
mod algorithm;
mod application;
mod data;
use clap::Parser;
use data::{Args, Data, State};
use std::path::PathBuf;

fn main() {
    let root_path: PathBuf = match current_exe() {
        Ok(path) => path,
        Err(err) => {
            panic!(
                "Fatal error. Cannot retrieve path of 'descramble.exe'. {}",
                err
            );
        }
    };
    let app: Application = Application::new(State::new(
        Args::parse(),
        Data::try_from(root_path.as_path())
            .expect("Fatal error. Data cannot be retrieved with Data::try_from"),
        root_path,
    ));
    app.start()
}

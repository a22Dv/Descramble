use application::Application;
use std::env::current_exe;
mod algorithm;
mod application;
mod data;
use clap::Parser;
use data::{Args, Data, State};
use std::path::{PathBuf};

/// DONE. No further changes.
/// Entry point.
fn main() {
    // Binary file path.
    let root_path: PathBuf = match current_exe() {
        Ok(path) => path.parent().unwrap().to_path_buf(),
        Err(err) => {
            panic!(
                "Fatal error. Cannot retrieve path of 'descramble.exe'.\n{}",
                err
            );
        }
    };
    let app: Application = Application::new(State::new(
        Args::parse(),
        match Data::try_from(&root_path.join("data")) {
            Ok(data) => data,
            Err(err) => panic!(
                "Fatal error. Cannot retrieve data using Data::TryFrom.\n{:?}",
                err
            ),
        },
        root_path.to_path_buf(),
    ));
    app.start()
}


mod app;
mod init;
use std::env::current_exe;
use std::path::PathBuf;
use app::Application;
use init::Args;

fn main() {
    let current_path: PathBuf = match current_exe() {
        Ok(path) => path,
        Err(err) => panic!("ERROR 01 - Unable to retrieve executable path. {}", err),
    };
    let args: Args = Args::parse();
    Application::new(current_path, args);
}

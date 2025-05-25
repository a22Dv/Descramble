mod app;
mod core;
mod data;
mod display;
use app::Application;
use clap::Parser;
use display::DisplayProgress;
use regex::Regex;
use std::env::current_exe;
use which::which;

#[derive(Clone, Parser, Debug)]
#[command(name = "descramble")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub anagram: String,

    #[arg(short = 'd', long = "depth", value_name = "LEVEL", value_parser = clap::value_parser!(u8).range(0..=10))]
    pub depth: Option<u8>,

    #[arg(short = 's', long = "show-top", value_name = "COUNT")]
    pub show_top: Option<usize>,

    #[arg(short = 'w', long = "word-count", value_name = "WORD_COUNT")]
    pub word_count: Option<usize>,

    #[arg(short = 'l', long = "llm-check")]
    pub llm_check: bool,
}

impl Args {
    pub fn new() -> Self {
        let mut args: Args = Args::parse();
        args.get_defaults();
        args
    }
    pub fn get_defaults(&mut self) {
        if self.depth == None {
            self.depth = Some(5u8);
        }
        if self.show_top == None {
            self.show_top = Some(50 as usize);
        }
        if self.word_count == None {
            self.word_count = Some(0 as usize)
        }
        if self.llm_check == true && which("llama-cli").is_err() {
            eprintln!("Error. `llama-cli` not found. Disabling LLM-based checks.");
            self.llm_check = false;
        }
    }
}
impl Default for Args {
    fn default() -> Self {
        Args::new()
    }
}

fn main() {
    DisplayProgress::clear_terminal();
    let app = Application::new(
        vec![Regex::new(r"^([a-z]|[A-Z]|\s)+$").unwrap()],
        match current_exe() {
            Ok(path) => path,
            Err(err) => {
                panic!("Fatal error. Cannot retrieve executable location. {}", err);
            }
        },
        Args::new(),
    );
    app.start();
}

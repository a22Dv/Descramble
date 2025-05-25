use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::{OwoColorize};
use clearscreen::clear;
use crate::Args;
pub struct DisplayProgress {
    bar: ProgressBar,
}
impl DisplayProgress {
    pub fn new(message: &str, bar_total: u64, unit: &str) -> Self {
        let bar_template: &str = "{msg}: [{elapsed_precise}] {bar:40.white/white} [{pos}/{len}]"; 
        let unit_template: &str = &format!("{} {{pos}}", unit);
        let progress_style: ProgressStyle = ProgressStyle::default_bar()
            .template(&format!("{} {}", bar_template, unit_template))
            .expect("Failed to parse.")
            .progress_chars("▪▪ ");
        let bar: ProgressBar = ProgressBar::new(bar_total)
            .with_message(message.to_string());
        bar.set_style(progress_style);
        DisplayProgress { 
            bar: bar, 
        }
    }
    pub fn increment(&self, by: u64) {
        self.bar.inc(by)
    }
    pub fn finish(&self) {
        self.bar.finish();
    }
    pub fn send_message(message: &str, rgb: (u8, u8, u8)) {
        let (r, g, b) = rgb;
        eprint!("{}", message.truecolor(r, g, b))
    }
    pub fn clear_terminal() {
        match clear() {
            Ok(_) => (),
            Err(_) =>  (),
        }
    }
}

pub fn display_solution(args: Args) {

}

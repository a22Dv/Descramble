use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "descramble")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    anagram: String,

    #[arg(short = 'f', long = "fast")]
    fast: bool,

    #[arg(short = 'd', long = "depth", value_name = "LEVEL", value_parser = clap::value_parser!(u8).range(0..=10))]
    depth: Option<u8>,

    #[arg(short = 's', long = "show-top", value_name = "COUNT")]
    show_top: Option<usize>,

    #[arg(short = 'w', long = "word-count", value_name = "WORD_COUNT")]
    word_count: Option<usize>,
}

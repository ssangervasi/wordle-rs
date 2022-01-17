#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use clap::{Parser, Subcommand};

fn main() {
    match cli() {
        Ok(_) => {}
        Err(msg) => eprintln!("{}", msg),
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short = 'n', long, default_value_t = 5)]
    word_len: usize,

    #[clap(short, long)]
    mkdict: bool,

    #[clap(short = 'i', long)]
    ui: bool,
}

fn cli() -> Result<(), String> {
    let args = Args::parse();

    if args.mkdict {
        wordle_rs::dicts::mkdict()
    } else if args.ui {
        wordle_rs::wordl::ui()
    } else {
        wordle_rs::wordl::play(
            &mut std::io::stdin(),
            &mut std::io::stdout(),
            wordle_rs::dicts::DICT.rand_of_len(args.word_len),
        )
    }
}

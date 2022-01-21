use clap::Parser;
use supports_unicode::Stream;

use wordle_rs::wordl::{Opts, play, ui};
use wordle_rs::dicts;

fn main() {
    match cli() {
        Ok(_) => {}
        Err(msg) => eprintln!("{}", msg),
    }
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(
        short = 'n',
        long,
        default_value_t = 5,
        help = "I dare you to try 2- or 10-letter words."
    )]
    word_len: usize,

    #[clap(
        short = 'l',
        long,
        help = "Play line-by-line instead of interactively."
    )]
    inline: bool,

    #[clap(short, long)]
    ascii: bool,

    #[clap(short, long)]
    unicode: bool,

    // //
    #[clap(short, long, hide = true)]
    mkdict: bool,

    #[clap(short = 'i', long, hide = true)]
    ui: bool,
}

fn cli() -> Result<(), String> {
    let args = Args::parse();

    let ascii = if args.unicode {
        false
    } else if args.ascii {
        true
    } else {
        !supports_unicode::on(Stream::Stdout)
    };

    if args.mkdict {
        dicts::mkdict()
    } else if args.inline {
        play(
            &mut std::io::stdin(),
            &mut std::io::stdout(),
            Opts {
                word_len: args.word_len,
                actual_raw: wordle_rs::dicts::DICT.rand_of_len(args.word_len),
                ascii,
            },
        )
    } else {
        ui(Opts {
            word_len: args.word_len,
            actual_raw: "".to_string(),
            ascii,
        })
    }
}

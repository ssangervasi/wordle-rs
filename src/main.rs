use clap::Parser;
use supports_unicode::Stream;

use wordle_rs::dicts;
use wordle_rs::wordl::{play, ui, Opts};

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

    #[clap(
        short = 'w',
        long,
        help = "The word you want to guess. (Maybe you set this up for someone else? Just testing things out?)"
    )]
    word: Option<String>,

    #[clap(
        short,
        long,
        help = "\
ASCII mode. Good for windows or if you're color blind like me :D
    x : Miss (not in word)
    ~ : Close (in word, wrong position)
    = : Match (in word at this position)\
"
    )]
    ascii: bool,

    #[clap(short, long)]
    unicode: bool,

    // --
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

    let (actual_raw, word_len) = if let Some(w) = args.word {
        (w.clone(), w.len())
    } else {
        (
            wordle_rs::dicts::DICT.rand_of_len(args.word_len),
            args.word_len,
        )
    };

    if args.mkdict {
        dicts::mkdict()
    } else if args.inline {
        play(
            &mut std::io::stdin(),
            &mut std::io::stdout(),
            Opts {
                word_len,
                actual_raw,
                ascii,
            },
        )
    } else {
        ui(Opts {
            word_len,
            actual_raw,
            ascii,
        })
    }
}

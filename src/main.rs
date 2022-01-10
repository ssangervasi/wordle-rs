#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use clap::{App, Arg};

fn main() {
    match cli() {
        Ok(_) => {}
        Err(msg) => eprintln!("{}", msg),
    }
}

fn cli() -> Result<(), String> {
    let matches = App::new("wordle-rs")
        .arg(Arg::from_usage("--mkdict"))
        .help("Wordle CLI")
        .get_matches();

    if matches.is_present("mkdict") {
        dicts::mkdict();
        return Ok(());
    }

    wordl::play(
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        dicts::rand_of_len(5),
    );

    Ok(())
}

pub mod wordl {
    use std::io;
    use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, Write};

    pub const MISS: char = '🟥';
    pub const MATCH: char = '🟩';
    pub const CLOSE: char = '🟨';

    pub fn compare(actual: &str, guess: &str) -> Vec<char> {
        let mut used: Vec<char> = actual.chars().collect();
        let mut res: Vec<char> = Vec::new();

        for (i, (ac, gc)) in actual.chars().zip(guess.chars()).enumerate() {
            if gc == ac {
                res.push(MATCH);
                used[i] = '-';
            } else {
                res.push(MISS);
            }
        }

        for (i, gc) in guess.chars().enumerate() {
            if res[i] == MATCH {
                continue;
            }

            let close_res = used.iter().position(|&uc| gc == uc);
            if let Some(ci) = close_res {
                res[i] = CLOSE;
                used[ci] = '-';
            }
        }

        res
    }

    #[test]
    fn test_compare() {
        {
            let res = compare("slump", "plump");
            println!("{}", res.iter().collect::<String>());
            assert_eq!(res, vec![MISS, MATCH, MATCH, MATCH, MATCH])
        }
        {
            let res = compare("slump", "maple");
            println!("{}", res.iter().collect::<String>());
            assert_eq!(res, vec![CLOSE, MISS, CLOSE, CLOSE, MISS])
        }
        {
            let res = compare("cacao", "anana");
            println!("{}", res.iter().collect::<String>());
            assert_eq!(res, vec![CLOSE, MISS, CLOSE, MISS, MISS])
        }
        {
            let res = compare("aquas", "pumas");
            println!("{}", res.iter().collect::<String>());
            assert_eq!(res, vec![MISS, CLOSE, MISS, MATCH, MATCH])
        }
    }

    pub fn play(input: &mut dyn Read, output: &mut dyn std::io::Write, actual: String) {
        writeln!(output, "Guess the five-letter word.").unwrap();

        let mut b = BufReader::new(input);

        let mut guesses = 6;
        let mut tolerance = 0;
        while guesses > 0 {
            let guess = read_trimmed(&mut b).to_uppercase();
            if guess.len() != 5 {
                tolerance += 1;
                if tolerance < 10 {
                    continue;
                } else {
                    panic!("Looks like you don't want to play.");
                }
            }

            guesses -= 1;
            tolerance = 0;

            for c in guess.chars() {
                write!(output, "{} ", c).unwrap();
            }
            writeln!(output).unwrap();
            let cmp = compare(&actual, &guess);
            writeln!(output, "{}", cmp.into_iter().collect::<String>()).unwrap();
        }
        writeln!(output, "Answer: {}", actual).unwrap();
    }

    pub fn read_trimmed<R: Read>(buf: &mut BufReader<R>) -> String {
        let mut response = String::new();
        buf.read_line(&mut response).expect("Failed to read line");
        response.trim().to_string()
    }

    #[test]
    fn test_play() {
        let mut input = Cursor::new(vec![]);
        input
            .write_all(
                b"
                    slink
                    trunk
                    ramps
                    pumps
                    plump
                    slump
            ",
            )
            .unwrap();
        input.rewind().unwrap();
        let mut expected = vec![
            vec![MATCH, MATCH, MISS, MISS, MISS],
            vec![MISS, MISS, MATCH, MISS, MISS],
            vec![MISS, MISS, CLOSE, CLOSE, CLOSE],
            vec![CLOSE, CLOSE, CLOSE, MISS, CLOSE],
            vec![MISS, MATCH, MATCH, MATCH, MATCH],
            vec![MATCH, MATCH, MATCH, MATCH, MATCH],
        ];

        let mut output = Cursor::new(vec![]);

        println!("Playing");
        play(&mut input, &mut output, "slump".to_string());
        println!("Done");

        output.rewind().unwrap();
        // let mut out_buf = BufReader::new(output);
        // let lines = out_buf.lines().collect();
        for line in output.lines().map(|l| l.unwrap()) {
            if line.chars().next().unwrap().is_alphabetic() {
                println!("{:?}", line);
                continue;
            }
            println!("{:?}", line);
            let expected_line = expected.remove(0).into_iter().collect::<String>();
            println!("{:?}", expected_line);

            assert_eq!(line, expected_line);
        }

        // assert_eq!(
        //     out_line,
        //     vec![CLOSE, MISS, CLOSE, CLOSE, MISS]
        //         .into_iter()
        //         .collect::<String>()
        // )
    }
}

pub mod dicts {
    use std::fs::{File, OpenOptions};
    use std::io::prelude::*;
    use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek, Write};

    use rand::seq::IteratorRandom;

    /**
     * Words, no punctuation, all caps.
     */
    pub fn dict() -> Vec<String> {
        let f = File::open("/usr/share/dict/american-english").unwrap();
        let reader = BufReader::new(f);

        let mut res: Vec<String> = Vec::new();

        for line_r in reader.lines() {
            let line_s = line_r.unwrap().trim().to_string();
            if line_s.chars().all(|c| c.is_alphabetic()) {
                res.push(line_s.to_uppercase());
            }
        }

        res
    }

    /**
     * Linux american english, removing words with:
     *  - punctuation (spaces, apostrophe, etc)
     *  - more than one uppercase letter (acronyms)
     *  - non-ascii (accents, etc.)
     */
    pub fn mkdict_loose() {
        let mut i = File::open("/usr/share/dict/american-english").unwrap();
        let mut o = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./dicts/loose")
            .unwrap();

        filter(&mut i, &mut o, |s| {
            if !s.chars().all(|c| c.is_ascii_alphabetic())
                || s.chars().filter(|c| c.is_uppercase()).count() > 1
            {
                None
            } else {
                Some(s.to_string())
            }
        })
    }

    /**
     * Loose, further limited to:
     *  - no than uppercase letters (proper nouns)
     */
    pub fn mkdict_improper() {
        let mut i = File::open("./dicts/loose").unwrap();
        let mut o = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./dicts/improper")
            .unwrap();

        filter(&mut i, &mut o, |s| {
            if s.chars().filter(|c| c.is_uppercase()).count() > 0 {
                None
            } else {
                Some(s.to_string())
            }
        })
    }

    /**
     * Improper, further limited to:
     *  - no words that are the previous word with an "s" added (pluarls)
     */
    pub fn mkdict_singular() {
        let mut i = File::open("./dicts/improper").unwrap();
        let mut o = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./dicts/singular")
            .unwrap();

        let mut prevs = std::collections::VecDeque::<String>::with_capacity(21);
        filter(&mut i, &mut o, |s| {
            if prevs
                .iter()
                .any(|w| s == format!("{}s", w) || s == format!("{}es", w))
                || s.ends_with("ies")
            {
                None
            } else {
                prevs.push_back(s.to_string());
                if prevs.len() > 20 {
                    prevs.pop_front();
                }
                Some(s.to_string())
            }
        })
    }

    pub fn mkdict() {
        mkdict_loose();
        mkdict_improper();
        mkdict_singular();
    }

    pub fn filter<F>(input: &mut dyn Read, output: &mut dyn std::io::Write, mut f: F)
    where
        F: FnMut(&str) -> Option<String>,
    {
        let ibuf = BufReader::new(input);
        let mut obuf = BufWriter::new(output);
        for line_r in ibuf.lines() {
            let line_f = f(&line_r.unwrap());
            if let Some(line_s) = line_f {
                writeln!(obuf, "{}", line_s).unwrap();
            }
        }
    }

    pub fn word_lens(len: usize) -> Vec<String> {
        dict().into_iter().filter(|w| w.len() == len).collect()
    }

    pub fn rand_of_len(len: usize) -> String {
        let mut rng = rand::thread_rng();
        word_lens(len).into_iter().choose(&mut rng).unwrap()
    }

    #[test]
    fn test_dict() {
        let d = dict();
        println!("Dict len: {:}", d.len());
        assert!(d.len() > 50_000);
    }

    #[test]
    fn test_lens() {
        let fives = word_lens(5);
        assert!(fives.into_iter().all(|w| w.len() == 5))
    }

    #[test]
    fn test_rand() {
        let w = rand_of_len(5);
        println!("W: {:?}", w);
        assert!(w.len() == 5)
    }
}

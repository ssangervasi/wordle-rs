pub mod ui;

pub mod wordl {
    #![allow(clippy::len_without_is_empty)]

    use std::io::{BufRead, BufReader, Read, Write};

    use crate::dicts::DICT;

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
            println!("{}", join(&res));
            assert_eq!(res, vec![MISS, MATCH, MATCH, MATCH, MATCH])
        }
        {
            let res = compare("slump", "maple");
            println!("{}", join(&res));
            assert_eq!(res, vec![CLOSE, MISS, CLOSE, CLOSE, MISS])
        }
        {
            let res = compare("cacao", "anana");
            println!("{}", join(&res));
            assert_eq!(res, vec![CLOSE, MISS, CLOSE, MISS, MISS])
        }
        {
            let res = compare("aquas", "pumas");
            println!("{}", join(&res));
            assert_eq!(res, vec![MISS, CLOSE, MISS, MATCH, MATCH])
        }
    }

    pub struct Game {
        actual: String,
        guesses: Vec<(String, Vec<char>)>,
    }

    pub const MAX_GUESSES: usize = 6;

    impl Game {
        pub fn new(actual: &str) -> Self {
            Self {
                actual: normalize(actual),
                guesses: Vec::with_capacity(6),
            }
        }

        pub fn with_len(len: usize) -> Self {
            Self::new(&crate::dicts::DICT.rand_of_len(len))
        }

        pub fn guess(&mut self, guess_raw: &str) -> Result<Vec<char>, String> {
            let guess = normalize(guess_raw);

            if self.guesses_remaining() < 1 {
                return Err(format!(
                    "No guesses remaining! {}",
                    if self.is_won() {
                        "You won!"
                    } else {
                        "You lost!"
                    },
                ));
            }
            if guess.len() != self.len() {
                return Err(format!("Guess {:?} not of length {}!", guess, self.len()));
            }
            if !crate::dicts::DICT.has(&guess) {
                return Err(format!("Guess {:?} not in word list!", guess));
            }

            let cmp = compare(&self.actual, &guess);
            self.guesses.push((guess, cmp.clone()));

            Ok(cmp)
        }

        pub fn len(&self) -> usize {
            self.actual.len()
        }

        pub fn guesses_made(&self) -> usize {
            self.guesses.len()
        }

        pub fn guesses_remaining(&self) -> usize {
            if self.is_won() {
                return 0;
            }

            MAX_GUESSES - self.guesses.len()
        }

        pub fn is_won(&self) -> bool {
            if let Some((_, cmp)) = self.guesses.last() {
                if cmp.iter().all(|&g| g == MATCH) {
                    return true;
                }
            }

            false
        }
    }

    #[test]
    fn test_game() {
        let actual = "slump";
        let guesses = vec![
            ("tight", vec![MISS, MISS, MISS, MISS, MISS]),
            ("slink", vec![MATCH, MATCH, MISS, MISS, MISS]),
            ("trunk", vec![MISS, MISS, MATCH, MISS, MISS]),
            ("chump", vec![MISS, MISS, MATCH, MATCH, MATCH]),
            ("plump", vec![MISS, MATCH, MATCH, MATCH, MATCH]),
            ("slump", vec![MATCH, MATCH, MATCH, MATCH, MATCH]),
        ];

        println!("Playing");
        let mut game = Game::new(actual);
        for (guess, expected) in guesses {
            let cmp = game.guess(guess);
            println!("{:?}\n\t{:?}\n\t{:?}", guess, cmp, expected);
            assert_eq!(cmp.unwrap(), expected);
        }
        println!("Done");
    }

    pub fn normalize(s: &str) -> String {
        s.to_uppercase()
    }

    pub fn join(cmp: &[char]) -> String {
        cmp.iter().collect::<String>()
    }

    pub fn play(
        input: &mut dyn Read,
        output: &mut dyn Write,
        actual_raw: String,
    ) -> Result<(), String> {
        let actual = normalize(&actual_raw);
        writeln!(output, "Guess the word of length {}.", actual.len()).unwrap();

        let mut b = BufReader::new(input);

        let mut guesses = 0;
        let mut tries = 0;
        while guesses < 6 {
            let guess = normalize(&read_trimmed(&mut b));
            let prev_tries = tries;
            if guess.len() != actual.len() {
                writeln!(output, "Guess must be {} letters ", actual.len()).unwrap();
                tries += 1;
            } else if !DICT.has(&guess) {
                writeln!(output, "Guess not in word list").unwrap();
                tries += 1;
            }

            if prev_tries != tries {
                if tries >= 10 {
                    return Err("Looks like you don't want to play.".to_string());
                } else {
                    continue;
                };
            }

            guesses += 1;
            tries = 0;

            for c in guess.chars() {
                write!(output, "{} ", c).unwrap();
            }
            writeln!(output).unwrap();
            let cmp = compare(&actual, &guess);
            // writeln!(output, "{}", cmp.into_iter().collect::<String>()).unwrap();
            writeln!(output, "{}", join(&cmp)).unwrap();
        }
        writeln!(output, "Answer: {}", actual).unwrap();

        Ok(())
    }

    pub fn read_trimmed<R: Read>(buf: &mut BufReader<R>) -> String {
        let mut response = String::new();
        buf.read_line(&mut response).expect("Failed to read line");
        response.trim().to_string()
    }

    #[test]
    fn test_play() {
        use std::io::prelude::*;
        use std::io::Cursor;

        let mut input = Cursor::new(vec![]);
        input
            .write_all(
                b"
                    tight
                    slink
                    trunk
                    chump
                    plump
                    slump
            ",
            )
            .unwrap();
        input.rewind().unwrap();
        let mut expected = vec![
            vec![MISS, MISS, MISS, MISS, MISS],
            vec![MATCH, MATCH, MISS, MISS, MISS],
            vec![MISS, MISS, MATCH, MISS, MISS],
            vec![MISS, MISS, MATCH, MATCH, MATCH],
            vec![MISS, MATCH, MATCH, MATCH, MATCH],
            vec![MATCH, MATCH, MATCH, MATCH, MATCH],
        ];

        let mut output = Cursor::new(vec![]);

        println!("Playing");
        play(&mut input, &mut output, "slump".to_string()).unwrap();
        println!("Done");

        output.rewind().unwrap();
        for line in output.lines().map(|l| l.unwrap()) {
            if line.chars().next().unwrap().is_alphabetic() {
                println!("{:?}", line);
                continue;
            }
            println!("{:?}", line);
            let expected_line = join(&expected.remove(0));
            println!("{:?}", expected_line);

            assert_eq!(line, expected_line);
        }
    }

    pub fn ui() -> Result<(), String> {
        use crate::ui::position::Position;
        // use crate::ui::screen;
        use crate::ui::term;
        use crate::ui::term::Res;

        let mut screen = term::default_screen();
        term::make_room();

        let mut game = Game::with_len(5);

        let prompt_start = Position::new(0, 0);
        let guesses_start = Position::new(0, prompt_start.row + 1);
        let guesses_end =
            guesses_start + Position::new(game.len() as i32 - 1, game.guesses_remaining() as i32);
        let err_start = Position::new(0, guesses_end.row + 1);
        screen.writes(
            &prompt_start,
            &format!(
                "\
Guess the word of length {}.
_____
_____
_____
_____
_____
_____

",
                game.len()
            ),
        );

        term::event_loop(|cursor, res| {
            let guess_start = guesses_start + Position::new(0, game.guesses_made() as i32);
            let guess_end = Position::new(guesses_end.row, guess_start.row);

            let handled = match res {
                Res::None => {
                    if cursor.col == 0 && cursor.row == 0 {
                        Res::Move((0, 1).into())
                    } else {
                        Res::None
                    }
                }
                Res::Write(ch) => {
                    screen.write(&cursor, ch.to_ascii_uppercase());

                    Res::Move(Position::new(1, 0).clamp(
                        //
                        (0 - cursor.col, 0).into(),
                        (guess_end.col - cursor.col, 0).into(),
                    ))
                }
                Res::Enter => {
                    let guess_raw = screen.reads(&guess_start, &guess_end);
                    let guess = guess_raw.chars().filter(|&c| c != '_').collect::<String>();

                    let res = game.guess(&guess);
                    match res {
                        Ok(cmp) => {
                            screen.writes(&(guess_end + (1, 0).into()), &join(&cmp));

                            if 0 < game.guesses_remaining() {
                                Res::Move((-cursor.col, 1).into())
                            } else {
                                Res::Quit
                            }
                        }
                        Err(msg) => {
                            screen.writes(&err_start, &msg);
                            Res::None
                        }
                    }
                }
                Res::Backspace => {
                    screen.write(&cursor, '_');

                    Res::Move((-1, 0).into())
                }
                Res::Move(dp) => {
                    Res::Move(dp.clamp(
                        //
                        (0 - cursor.col, 0).into(),
                        (guess_end.col - cursor.col, 0).into(),
                    ))
                }
                _ => res,
            };
            term::just_dump_screen(&mut screen).unwrap();
            handled
        })
        .unwrap();

        Ok(())
    }
}

pub mod dicts {
    use std::fs::{File, OpenOptions};
    use std::io::{BufRead, BufReader, BufWriter, Read, Write};

    use lazy_static::lazy_static;
    use rand::seq::IteratorRandom;

    static RAW_DICT: &str = include_str!("../dicts/singular");
    lazy_static! {
        pub static ref DICT: Dict = Dict::new();
    }

    pub fn dict_words() -> Vec<String> {
        let mut res: Vec<String> = Vec::new();

        for line_r in RAW_DICT.lines() {
            let line_s = line_r.trim().to_string();
            if !line_s.is_empty() {
                res.push(line_s.to_uppercase());
            }
        }

        res
    }

    pub struct Dict {
        words: Vec<String>,
    }

    impl Default for Dict {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Dict {
        pub fn new() -> Self {
            Self {
                words: dict_words(),
            }
        }

        pub fn word_lens(&self, len: usize) -> Vec<String> {
            self.words
                .clone()
                .into_iter()
                .filter(|w| w.len() == len)
                .collect()
        }

        pub fn rand_of_len(&self, len: usize) -> String {
            let mut rng = rand::thread_rng();
            self.word_lens(len).into_iter().choose(&mut rng).unwrap()
        }

        pub fn has(&self, word: &str) -> bool {
            self.words.binary_search(&word.to_string()).is_ok()
        }
    }

    /**
     * Linux american english, removing words with:
     *  - punctuation (spaces, apostrophe, etc)
     *  - more than one uppercase letter (acronyms)
     *  - non-ascii (accents, etc.)
     */
    fn mkdict_loose() {
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
    fn mkdict_improper() {
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
    fn mkdict_singular() {
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

    pub fn mkdict() -> Result<(), String> {
        mkdict_loose();
        mkdict_improper();
        mkdict_singular();
        Ok(())
    }

    fn filter<F>(input: &mut dyn Read, output: &mut dyn std::io::Write, mut f: F)
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

    #[test]
    fn test_dict() {
        let d = &crate::dicts::DICT;
        println!("Dict len: {:}", d.words.len());
        assert!(d.words.len() > 40_000);
    }

    #[test]
    fn test_lens() {
        let fives = crate::dicts::DICT.word_lens(5);
        assert!(fives.into_iter().all(|w| w.len() == 5))
    }

    #[test]
    fn test_rand() {
        let w = crate::dicts::DICT.rand_of_len(5);
        println!("W: {:?}", w);
        assert!(w.len() == 5)
    }
}

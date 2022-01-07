use clap::App;
use std::io;

fn main() {
    match cli() {
        Ok(_) => {}
        Err(msg) => eprintln!("{}", msg),
    }
}

fn cli() -> Result<(), String> {
    let _matches = App::new("wordle-rs").help("Wordle CLI").get_matches();

    play();

    Ok(())
}

fn play() {
    loop {
        let s = read_trimmed();
        eprintln!("{:?}", s);
    }
}

fn read_trimmed() -> String {
    let mut response = String::new();
    io::stdin()
        .read_line(&mut response)
        .expect("Failed to read line");
    response.trim().to_string()
}

fn compare(actual: &str, guess: &str) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for (ac, gc) in actual.chars().zip(guess.chars()) {
        if ac == gc {
            res.push(format!("{}+", gc));
        } else {
            res.push(format!("{}-", gc));
        }
    }
    res
}

#[test]
fn test_compare() {
    let res = compare("slump", "plump");
    assert_eq!(res, vec!["p-", "l+", "u+", "m+", "p+",])
}

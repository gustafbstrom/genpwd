use std::io::{BufRead,BufReader};
use std::fs::File;
use rand::Rng;
use std::path::Path;
use clap;

fn import_word_list() -> Vec<String> {
    let path = Path::new("words_alpha.txt");
    let display = path.display();
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let buffered = BufReader::new(file);
    
    buffered
        .lines()
        .map(|word| word.unwrap())
        .collect()
}

fn generate_pass(n_words: u32, prefix: String, suffix: String) -> String {
    let word_list = import_word_list();
    let mut rng = rand::thread_rng();
    let mut pass = prefix.clone();
    for _ in 0..n_words {
        let r: usize = rng.gen::<usize>() % word_list.len();
        let mut tmp_str = word_list[r].clone();
        if let Some(r) = tmp_str.get_mut(0..1) {
            r.make_ascii_uppercase();
        }
        pass += &tmp_str.to_string();
    }
    pass += &suffix;
    
    pass
}

fn parse_args() -> clap::ArgMatches<'static> {
    clap::App::new("genpwd")
        .version("1.0")
        .author("Gustaf Borgstrom <gustaf.borgstrom@koltrast.se>")
        .about("Generate easy to comprehend, hard to crack passwords.")
        .arg(clap::Arg::with_name("n_words")
            .short("w")
            .long("words")
            .help("Number of words to include")
            .required(false)
            .default_value("3"))
        .arg(clap::Arg::with_name("prefix")
            .long("prefix")
            .help("Fixed prefix before the generated password")
            .required(false)
            .default_value("131"))
        .arg(clap::Arg::with_name("suffix")
            .long("suffix")
            .help("Fixed suffix after the generated password")
            .required(false)
            .default_value("."))
        .get_matches()
}
    
fn main() {
    let args = parse_args();
    let words = args
        .value_of("n_words")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let pass = generate_pass(
        words,
        args.value_of("prefix").unwrap().replace(" ", "_"),
        args.value_of("suffix").unwrap().replace(" ", "_")
    );
    println!("{}", pass);
}

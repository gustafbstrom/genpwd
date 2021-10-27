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

struct PassGen {
    word_list: Vec<String>,
    rng: rand::rngs::ThreadRng,
}

impl PassGen {
    pub fn new() -> Self {
        Self {
            word_list: import_word_list(),
            rng: rand::thread_rng(),
        }
    }

    fn generate_words(&mut self, n_words: u32) -> String {
        let mut pass = String::new();
        for _ in 0..n_words {
            let i = self.rng.gen::<usize>() % self.word_list.len();
            let mut tmp_str = self.word_list[i].clone();
            if let Some(r) = tmp_str.get_mut(0..1) {
                r.make_ascii_uppercase();
            }
            pass += &tmp_str;
        }
        pass
    }

    pub fn generate_pass(&mut self, n_words: u32, prefix: Option<String>, suffix: Option<String>) -> String {
        let mut pass = String::new();
        if let Some(s) = prefix {
            pass += &s;
        }
        pass += &self.generate_words(n_words);
        if let Some(s) = suffix {
            pass += &s;
        }
        pass
    }
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
        .arg(clap::Arg::with_name("qrcode")
            .short("q")
            .long("qr-code")
            .help("Generate and display a QR code representation of the generated pass")
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

    let mut pwd_gen = PassGen::new();
    let pass = pwd_gen.generate_pass(
        words,
        Some(args.value_of("prefix").unwrap().replace(" ", "_")),
        Some(args.value_of("suffix").unwrap().replace(" ", "_")),
    );
    println!("{}", pass);
}

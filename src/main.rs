use std::io;
use std::io::*;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use rand::Rng;
use clap::{self,ArgMatches};

#[cfg(feature = "default")]
use sdl2::event::Event;

#[cfg(feature = "default")]
use qr_code;

fn import_word_list(path: &str) -> Vec<String> {
    let path = format!("{}/wordlist", path);
    let path = Path::new(path.as_str());
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

fn parse_config(path: &str) -> HashMap<String, String> {
    let path = format!("{}/config", path);
    let path = Path::new(&path);
    let display = path.display();
    let file = match File::open(&path) {
        Err(why) => return HashMap::new(),
        Ok(file) => file,
    };

    let buffered = BufReader::new(file);
    let lines : Vec<String> = buffered
        .lines()
        .map(|word| word.unwrap()) //.split("=").collect())
        .collect();
    
    let key_vals : Vec<Vec<String>> = lines
        .into_iter()
        .map(|word| word.split("=").map(|s| String::from(s)).collect())
        .collect();

    let mut hm = HashMap::new();
    for (idx, key_val) in key_vals.iter().enumerate() {
        if key_val.len() != 2 {
            panic!("Malformed config on line {}", idx);
        }
        
        let key = String::from(key_val[0].trim().clone());
        let val = String::from(key_val[1].trim().clone());
        if hm.contains_key(&key) {
            panic!("Double config on line {}", idx);
        }
        hm.insert(key, val);
    }

    hm
}

struct PassGen {
    word_list: Vec<String>,
    rng: rand::rngs::ThreadRng,
}

impl PassGen {
    pub fn new(wl_path: &str) -> Self {
        Self {
            word_list: import_word_list(wl_path),
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

    pub fn generate_pass(&mut self, n_words: u32, prefix: &Option<String>, suffix: &Option<String>) -> String {
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

    pub fn get_user_input(&mut self) -> bool {
        let mut input = String::new();
        let stdin = io::stdin();
        loop {
            print!("Ok? (y/n): ");
            io::stdout().flush().unwrap();
            stdin
                .read_line(&mut input)
                .expect("Error: unable to read user input");
            match input.trim() {
                "y" => return true,
                "n" => return false,
                _ => (),
            }
            input.clear();
        }
    }
}

#[cfg(feature = "default")]
fn gen_qr_code(pass: &str) -> qr_code::QrCode {
    let qc = qr_code::QrCode::new(pass);
    qc.unwrap()
}

#[cfg(feature = "default")]
fn show_qr_code(qc: &qr_code::QrCode) {
    let mut qc_v = Vec::new();
    qc.to_bmp().write(&mut qc_v).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("pwdgen", 320, 320)
        .position_centered()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string()).unwrap();
    let texture_creator = canvas.texture_creator();
    let mut bmprwops = sdl2::rwops::RWops::from_bytes(&qc_v).unwrap();
    let texture = sdl2::surface::Surface::load_bmp_rw(&mut bmprwops)
        .unwrap()
        .as_texture(&texture_creator)
        .unwrap();

    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }
    }
}

fn parse_args() -> clap::ArgMatches<'static> {
    let mut arg_build = clap::App::new("genpwd")
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
            .required(false))
        .arg(clap::Arg::with_name("suffix")
            .long("suffix")
            .help("Fixed suffix after the generated password")
            .required(false))
        .arg(clap::Arg::with_name("shared_path")
            .long("shared-path")
            .help("Specifies where to find shared files")
            .required(false))
        .arg(clap::Arg::with_name("interactive")
            .short("i")
            .long("interactive")
            .help("Interactive response of accepting the generated pass")
            .required(false));

    #[cfg(feature = "default")]
    {
        arg_build = arg_build
            .arg(clap::Arg::with_name("qrcode")
            .short("q")
            .long("qr-code")
            .help("Generate and display a QR code representation of the generated pass")
            .required(false)
            .takes_value(false));
    }

    arg_build.get_matches()
}
    
fn main() {
    fn get_conf_val(args: &ArgMatches,
                    config: &HashMap<String, String>,
                    key: &str) -> Option<String> {
        if args.is_present(key) {
            Some(args.value_of(key).unwrap().replace(" ", "_"))
        }
        else if config.contains_key(key.to_uppercase().as_str()) {
            let val = config.get(key.to_uppercase().as_str()).unwrap();
            Some(String::from(val)) // Already wrapped in an Option
        }
        else {
            None
        }
    }
    let args = parse_args();
    let words = args
        .value_of("n_words")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let mut pass: String;
    let shared_path = format!("{}/.config/genpwd",
                       std::env::var("HOME").unwrap());
    let config = parse_config(shared_path.as_str());
    let prefix = get_conf_val(&args, &config, "prefix");
    let suffix = get_conf_val(&args, &config, "suffix");
    let mut wl_path = get_conf_val(&args, &config, "shared_path");
    if wl_path.is_none() {
        wl_path = Some(shared_path);
    }
    let mut pwd_gen = PassGen::new(wl_path.unwrap().as_str());

    loop {
        pass = pwd_gen.generate_pass(words, &prefix, &suffix);
        println!("{}", pass);
        if !args.is_present("interactive") || pwd_gen.get_user_input() {
            break;
        }
        pass.clear();
    }

    #[cfg(feature = "default")]
    if args.is_present("qrcode") {
        let qc = gen_qr_code(&pass);
        show_qr_code(&qc);
    }
}

use std::io::{Read, BufReader};
use std::fs::File;
use std::path::Path;
use clap;
use toml::Table;
use crate::passgen::passgen::PassGen;

pub mod passgen;

#[cfg(feature = "default")]
use sdl2::event::Event;

#[cfg(feature = "default")]
use qr_code;

fn parse_config(path: &str) -> toml::map::Map<String, toml::Value> {
    let path = format!("{}/config.toml", path);
    let path = Path::new(&path);
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => return toml::map::Map::new(),
    };

    let mut buffered = BufReader::new(file);
    let mut buf = String::new();
    buffered.read_to_string(&mut buf).unwrap();
    let toml_config = buf.parse::<Table>().unwrap();

    toml_config
}

#[cfg(feature = "default")]
fn gen_qr_code(pass: &str) -> qr_code::QrCode {
    let qc = qr_code::QrCode::new(pass);
    qc.unwrap()
}

#[cfg(feature = "default")]
fn render_display_qr_code(qc: &qr_code::QrCode) {
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

fn parse_args() -> clap::ArgMatches {
    let mut arg_build = clap::Command::new("genpwd")
        .version("1.0")
        .author("Gustaf Borgstrom <gustaf.borgstrom@koltrast.se>")
        .about("Generate easy to comprehend, hard to crack passwords.")
        .arg(clap::Arg::new("n_words")
            .short('w')
            .long("words")
            .help("Number of words to include")
            .required(false)
            .value_parser(clap::value_parser!(u32))
            .default_value("3")
        )
        .arg(clap::Arg::new("prefix")
            .long("prefix")
            .help("Fixed prefix before the generated password")
            .required(false)
        )
        .arg(clap::Arg::new("suffix")
            .long("suffix")
            .help("Fixed suffix after the generated password")
            .required(false)
        )
        .arg(clap::Arg::new("shared_path")
            .long("shared-path")
            .help("Specifies where to find shared files")
            .required(false)
        )
        .arg(clap::Arg::new("interactive")
            .short('i')
            .long("interactive")
            .help("Interactive response of accepting the generated pass")
            .required(false)
            .action(clap::ArgAction::SetTrue)
        );

    #[cfg(feature = "default")]
    {
        arg_build = arg_build
            .arg(clap::Arg::new("qrcode")
            .short('q')
            .long("qr-code")
            .help("Generate and display a QR code representation of the generated pass")
            .required(false)
            .action(clap::ArgAction::SetTrue)
        );
    }

    arg_build.get_matches()
}

fn run() {
    // Precedence: cmd line flag, else config file, else none
    fn get_conf_val(args: &clap::ArgMatches,
                    config: &toml::map::Map<String, toml::Value>,
                    key: &str) -> Option<String> {
        if args.contains_id(key) {
            Some(args.get_one::<String>(key).unwrap().replace(" ", "_"))
        }
        else if config.contains_key(key.to_uppercase().as_str()) {
            let val = config
                .get(key.to_uppercase().as_str())
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            Some(val)
        }
        else {
            None
        }
    }

    let args = parse_args();

    let n_words = *args.get_one::<u32>("n_words").unwrap();

    // TODO: this might be better to not hardcode or at least respect different
    // OS:es and systems
    let home_path = std::env::var("HOME").unwrap();
    let shared_path = std::path::Path::new(&home_path)
        .join(".config")
        .join("genpwd");
    let config = parse_config(shared_path.to_str().unwrap());

    // Populate the password generator with the word list
    let wl_path = get_conf_val(&args, &config, "shared_path")
        .unwrap_or_else(|| shared_path.to_str().unwrap().to_string());
    let mut pwd_gen = PassGen::new(&wl_path);

    // Generate the password(s)
    let prefix = get_conf_val(&args, &config, "prefix");
    let suffix = get_conf_val(&args, &config, "suffix");
    let is_interactive = args.get_one::<bool>("interactive").unwrap();
    loop {
        pwd_gen.generate_new_pass(n_words, &prefix, &suffix);
        let new_pass = pwd_gen.view_current_pass();
        println!("{}", new_pass);
        if !is_interactive || pwd_gen.get_user_input() {
            break;
        }
    }

    #[cfg(feature = "default")]
    {
        let show_qr_code = *args.get_one::<bool>("qrcode").unwrap();
        if show_qr_code {
            let qc = gen_qr_code(pwd_gen.view_current_pass());
            render_display_qr_code(&qc);
        }
    }
}

fn main() {
    run()
}

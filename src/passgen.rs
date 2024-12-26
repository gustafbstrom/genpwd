pub mod passgen {
    use rand::Rng;
    use std::io;
    use std::io::*;
    use std::path::Path;
    use std::fs::File;

    fn import_word_list(path: &str) -> Vec<String> {
        let path = Path::new(path).join("wordlist");
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

    pub struct PassGen {
        word_list: Vec<String>,
        rng: rand::rngs::ThreadRng,
        current_pass: String,
    }

    impl PassGen {
        pub fn new(wl_path: &str) -> Self {
            Self {
                word_list: import_word_list(wl_path),
                rng: rand::thread_rng(),
                current_pass: String::new(),
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

        pub fn generate_new_pass(&mut self, n_words: u32, prefix: &Option<String>, suffix: &Option<String>) {
            let prefix_str = match prefix {
                Some(s) => s,
                None => "",
            };
            let suffix_str = match suffix {
                Some(s) => s,
                None => "",
            };

            self.current_pass = prefix_str.to_owned()
                + &self.generate_words(n_words)
                + suffix_str;
        }

        pub fn view_current_pass(&self) -> &str {
            &self.current_pass
        }

        // TODO: consider moving this out, it does not look like it belongs here
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
}

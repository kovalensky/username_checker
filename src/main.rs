#![allow(dead_code, unused_variables)]

use reqwest::blocking::get as http_get;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process::{exit, Command};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use win32console::console::WinConsole;

struct App {
    args: Vec<String>,
    config: Config,
}

#[derive(Deserialize)]
struct Config {
    usernames: HashMap<String, String>,
    api_key: String,
    bot_id: u32,
    user_id: u32,
    request_time: u64,
    queue_time: u64,
}

impl App {
    fn new() -> Self {
        let config_path = "config.toml";
        if !fs::metadata(&config_path).is_ok() {
            println!("Can't find config.toml in executable directory");
            exit(1);
        }

        let mut config_file = File::open(&config_path).unwrap();
        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).unwrap();

        let config: Config =
            toml::from_str(&config_string).expect("config.toml structure is not right");

        App {
            config,
            args: env::args().collect(),
        }
    }

    fn run(&mut self) {
        if self.args.len() > 1 {
            if self.args[1] == "hide" {
                self.start_hidden();
            } else if self.args[1] == "hidden" {
                WinConsole::free_console().unwrap();
            }
        }

        self.app();
    }

    fn start_hidden(&self) {
        let mut command = Command::new(&self.args[0]);
        command
            .args(["hidden"])
            .spawn()
            .expect("Failed to start hidden process");
        exit(0);
    }

    fn app(&self) {
        let t_api = format!(
            "https://api.telegram.org/bot{}:{}/sendMessage?chat_id={}",
            self.config.bot_id, self.config.api_key, self.config.user_id
        );
        let mut body = String::new();

        'outer: loop {
            for (username, mask) in &self.config.usernames {
                body.clear();

                match http_get(format!("https://t.me/{username}")) {
                    Ok(mut response) => {
                        self.stdout("Query for t.me is successful");
                        response.read_to_string(&mut body).unwrap();
                    }
                    _ => {
                        self.stdout("t.me query Error re-trying in 3s");
                        sleep(Duration::from_secs(3));
                        continue 'outer;
                    }
                }

                if !body.contains(mask) {
                    match http_get(format!("{t_api}&text=@{username} -> Frei!")) {
                        Ok(_) => self.stdout(&format!("{username} is free, sent the notification via bot!")),
                        _ => {
                            self.stdout(
                                &format!("{username} is free, but there was an Error contacting bot api"),
                            );
                            continue 'outer;
                        }
                    }
                }

                if self.config.usernames.len() > 1 {
                    sleep(Duration::from_secs(self.config.queue_time));
                }
            }
            sleep(Duration::from_secs(self.config.request_time));
        }
    }

    fn stdout(&self, msg: &str) {
        if self.args.len() == 1 || self.args[1] != "hidden" {
            println!("{msg}");
        }
    }
}

fn main() {
    let mut app = App::new();
    app.run();
}

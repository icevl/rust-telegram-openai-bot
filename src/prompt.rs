use db::DB;
use std::{
    io::{self, Write},
    iter, process,
    sync::{Arc, Mutex},
};

use crate::{db, utils::State};

const PROMPT: &str = "tg-bot CLI# ";

pub fn prompt_process(state: Arc<Mutex<State>>) {
    let db = DB::new();
    let mut buffer = String::new();

    loop {
        print!("{}", PROMPT);
        io::stdout().flush().expect("failed to flush stdout");

        io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read line");

        if buffer.trim().is_empty() {
            buffer.clear();
            continue;
        }

        if buffer.starts_with("hello") {
            println!("Hello, world!");
        } else if buffer.starts_with("reload") {
            let users_list = db.get_users().unwrap();
            state.lock().unwrap().users = Mutex::new(users_list);
            println!("Users database reloaded!");
        } else if buffer.starts_with("exit") {
            process::exit(0);
        } else {
            println!("Unknown command");
        }

        print!("{}", iter::repeat('\r').take(0).collect::<String>());

        buffer.clear();
    }
}

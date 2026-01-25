#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("failed to read command");
    
        let command = command.trim();

        if command == "exit" {
            break;
        }

        let words: Vec<&str> = command.split(' ').collect();

        if !words.is_empty() && words[0] == "echo" {
            let args = words[1..].join(" ");
            println!("{args}");
        } else {
            println!("{command}: command not found");
        }
    }
}

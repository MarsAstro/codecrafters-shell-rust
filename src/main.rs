use std::io::{self, Write};
use std::env;

const BUILTINS: &[&str] = &["exit", "echo", "type"];

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "exit" => std::process::exit(0),
            "echo" => {
                println!("{}", args.join(" "));
            }
            " " => continue,
            "type" => {
                if args.is_empty() {
                    println!("type: missing operand");
                    continue;
                }

                let target = args[0];

                if BUILTINS.contains(&target) {
                    println!("{} is a shell builtin", target);
                } else if let Some(path) = find_executable_in_path(target) {
                    println!("{} is {}", target, path);
                } else {
                    println!("{target}: not found")
                }
            }
            _ => println!("{}: command not found", cmd)
        }
    }
}

fn find_executable_in_path(file_name: &str) -> Option<String> {
    match env::var_os("PATH") {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                let full_path = path.join(file_name);

                if std::path::Path::new(&full_path).exists() {
                    #[cfg(unix)] {

                    if let Ok(metadata) = std::fs::metadata(&full_path) {
                        let permissions = metadata.permissions();
                        use std::os::unix::fs::PermissionsExt;
                        if permissions.mode() & 0o111 != 0 {
                            return Some(full_path.display().to_string())
                        }
                    }
                    } #[cfg(not(unix))] {

                    return Some(full_path.display().to_string())

                    }
                }
            }
        }
        None => return None
    }

    None
}
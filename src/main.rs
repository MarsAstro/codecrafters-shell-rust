use std::io::{self, Write};
use std::env;
use std::process;

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

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
            " "     => continue,
            "exit"  => process::exit(0),
            "echo"  => println!("{}", args.join(" ")),
            "type"  => run_type_command(args),
            "pwd"   => run_pwd_command(),
            "cd"    => run_cd_command(args),
            _       => try_execute_command(cmd, args),
        }
    }
}

fn run_cd_command(args: &[&str]) {
    let arg = args[0];
    if std::path::Path::new(&arg).exists() {
        env::set_current_dir(arg).expect("Failed to change working directory");
    } else if arg == "~" {
        if let Some(path) = env::var_os("HOME") {
            env::set_current_dir(path).expect("Failed to change working directory");
        } else {
            println!("cd: No home directory configured in environment variables");
        }
    } else {
        println!("cd: {}: No such file or directory", arg);
    }
}

fn run_pwd_command() {
    if let Ok(cwd) = env::current_dir() {
        println!("{}", cwd.display());
    }
}

fn run_type_command(args: &[&str]) {
    if args.is_empty() {
        println!("type: missing operand");
        return;
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

fn try_execute_command(cmd: &str, args: &[&str]) {
    if let Some(_path) = find_executable_in_path(cmd) {
        let output = process::Command::new(cmd).args(args).output().expect("failed to execute program");
        io::stdout().write_all(&output.stdout).expect("couldn't write output to stdout");
    } else { 
        println!("{}: command not found", cmd);
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
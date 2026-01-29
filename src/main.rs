use std::fs::File;
use std::io::{self, Write};
use std::env;
use std::process::{self};
use shlex;

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

        // Advanced split that heeds standard shell single and double quote rules
        let args = shlex::split(command).unwrap();
        let args = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let mut args: Vec<&str> = args.iter().map(|s| &**s).collect();

        // If output redirected to file, store file name and remove redirect from args
        let mut output_file_path = String::new();
        if args.contains(&">") {
            let index = args.iter().position(|arg| arg.contains(">")).unwrap();
            if args.len() > index+1 {
                output_file_path = args[index+1].to_string();
                args.drain(index..);
            }
        } else if args.contains(&"1>") {
            let index = args.iter().position(|arg| arg.contains("1>")).unwrap();
            if args.len() > index+1 {
                output_file_path = args[index+1].to_string();
                args.drain(index..);
            }
        }

        // Separating first element into command and remaining elements as arguments to command
        let cmd = args[0];
        let args = &args[1..];
        
        let result: Result<String, String>;
        match cmd {
            " "     => continue,
            "exit"  => process::exit(0),
            "echo"  => result = run_echo_command(args),
            "type"  => result = run_type_command(args),
            "pwd"   => result = run_pwd_command(),
            "cd"    => result = run_cd_command(args),
            _       => result = try_execute_command(cmd, args),
        }

        match result {
            Ok(message) => {
                if !output_file_path.is_empty() {
                    write_output_to_file(message, output_file_path);
                } else if !message.is_empty() {
                    println!("{message}");
                }
            }
            Err(message) => {
                eprintln!("{message}");
            }
        }
    }
}

fn run_echo_command(args: &[&str]) -> Result<String, String> {
    let message = args.join(" ");
    Ok(message)
}

fn run_type_command(args: &[&str]) -> Result<String, String> {
    if args.is_empty() {
        let message = String::from("type: missing operand");
        return Err(message);
    }

    let target = args[0];

    let result: Result<String, String>;
    if BUILTINS.contains(&target) {
        let message = format!("{} is a shell builtin", target);
        result = Ok(message)
    } else if let Some(path) = find_executable_in_path(target) {
        let message = format!("{} is {}", target, path);
        result = Ok(message)
    } else {
        let message = format!("{target}: not found");
        result = Err(message)
    }

    result
}

fn run_pwd_command() -> Result<String, String> {
    let result: Result<String, String>;
    if let Ok(cwd) = env::current_dir() {
        let message = format!("{}", cwd.display());
        result = Ok(message)
    } else {
        let message = format!("no current working directory found");
        result = Err(message)
    }

    result
}

fn run_cd_command(args: &[&str]) -> Result<String, String> {
    let result: Result<String, String>;
    
    let arg = args[0];
    if std::path::Path::new(&arg).exists() {
        match env::set_current_dir(arg) {
            Ok(_) => result = Ok(String::new()),
            Err(_) => result = Err(String::from("failed to change working directory")),
        }
    } else if arg == "~" {
        if let Some(path) = env::var_os("HOME") {
            match env::set_current_dir(path) {
                Ok(_) => result = Ok(String::new()),
                Err(_) => result = Err(String::from("failed to change working directory")),
        }
        } else {
            let message = String::from("cd: No home directory configured in environment variables");
            result = Err(message);
        }
    } else {
        let message = format!("cd: {}: No such file or directory", arg);
        result = Err(message);
    }

    result
}

fn try_execute_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let result: Result<String, String>;

    if let Some(_path) = find_executable_in_path(cmd) {
        if let Ok(output) = process::Command::new(cmd).args(args).output() {
            if output.status.success() {
                match str::from_utf8(&output.stdout) {
                    Ok(val) => result = Ok(val.trim().to_string()),
                    Err(_) => result = Err(String::from("failed to parse program output as text"))
                }
            } else {
                match str::from_utf8(&output.stderr) {
                    Ok(val) => result = Err(val.trim().to_string()),
                    Err(_) => result = Err(String::from("failed to parse program output as text"))
                }
            }
        } else {
            let err_message = String::from("couldn't execute program");
            result = Err(err_message)
        }
    } else { 
        let err_msg = format!("{}: command not found", cmd);
        result = Err(err_msg);
    }

    result
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

fn write_output_to_file(output: String, file_path: String) {
    if let Ok(mut file) = File::create(file_path) {
        file.write(output.as_bytes()).expect("failed to write to file");
    }
}
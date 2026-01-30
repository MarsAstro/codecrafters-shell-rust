use std::fs::File;
use std::io::{self, Write};
use std::env;
use std::process::{self};
use shlex;

const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

struct CmdOutput {
    output: String,
    is_err: bool,
}

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

        // If stdout redirected to file, store file name and remove redirect from args
        let mut stdout_file_path = String::new();
        if args.contains(&">") {
            let index = args.iter().position(|arg| arg.contains(">")).unwrap();
            if args.len() > index+1 {
                stdout_file_path = args[index+1].to_string();
                args.drain(index..);
            }
        } else if args.contains(&"1>") {
            let index = args.iter().position(|arg| arg.contains("1>")).unwrap();
            if args.len() > index+1 {
                stdout_file_path = args[index+1].to_string();
                args.drain(index..);
            }
        }

        // If stderr redirected to file, store file name and remove redirect from args
        let mut stderr_file_path = String::new();
        if args.contains(&"2>") {
            let index = args.iter().position(|arg| arg.contains("2>")).unwrap();
            if args.len() > index+1 {
                stderr_file_path = args[index+1].to_string();
                args.drain(index..);
            }
        } 

        // Separating first element into command and remaining elements as arguments to command
        let cmd = args[0];
        let args = &args[1..];
        
        let mut output: Vec<CmdOutput> = Vec::new();
        match cmd {
            " "     => continue,
            "exit"  => process::exit(0),
            "echo"  => run_echo_command(&mut output, args),
            "type"  => run_type_command(&mut output, args),
            "pwd"   => run_pwd_command(&mut output),
            "cd"    => run_cd_command(&mut output, args),
            _       => try_execute_command(&mut output, cmd, args),
        }
        
        for elem in output {
            if !elem.is_err {
                if !stdout_file_path.is_empty() {
                    write_output_to_file(&elem.output, &stdout_file_path);
                } else {
                    println!("{}", elem.output);
                }
            } else {
                if !stderr_file_path.is_empty() {
                    write_output_to_file(&elem.output, &stderr_file_path);
                } else {
                    eprintln!("{}", elem.output);
                }
            }
        }
    }
}

fn run_echo_command(output: &mut Vec<CmdOutput>, args: &[&str]) {
    let message = args.join(" ");
    output.push(CmdOutput {output: message, is_err: false })
}

fn run_type_command(output: &mut Vec<CmdOutput>, args: &[&str]) {
    if args.is_empty() {
        let message = String::from("type: missing operand");
        output.push(CmdOutput {output: message, is_err: true });
        return;
    }

    let target = args[0];

    if BUILTINS.contains(&target) {
        let message = format!("{} is a shell builtin", target);
        output.push(CmdOutput {output: message, is_err: false });
    } else if let Some(path) = find_executable_in_path(target) {
        let message = format!("{} is {}", target, path);
        output.push(CmdOutput {output: message, is_err: false });
    } else {
        let message = format!("{target}: not found");
        output.push(CmdOutput {output: message, is_err: true });
    }
}

fn run_pwd_command(output: &mut Vec<CmdOutput>) {
    if let Ok(cwd) = env::current_dir() {
        let message = format!("{}", cwd.display());
        output.push(CmdOutput {output: message, is_err: false });
    } else {
        let message = format!("no current working directory found");
        output.push(CmdOutput {output: message, is_err: true });
    }
}

fn run_cd_command(output: &mut Vec<CmdOutput>, args: &[&str]) {
    let arg = args[0];
    if std::path::Path::new(&arg).exists() {
        match env::set_current_dir(arg) {
            Ok(_) => (),
            Err(_) => output.push(CmdOutput {output: String::from("failed to change working directory"), is_err: true }),
        }
    } else if arg == "~" {
        if let Some(path) = env::var_os("HOME") {
            match env::set_current_dir(path) {
                Ok(_) => (),
                Err(_) => output.push(CmdOutput {output: String::from("failed to change working directory"), is_err: true }),
        }
        } else {
            let message = String::from("cd: No home directory configured in environment variables");
            output.push(CmdOutput {output: message, is_err: true });
        }
    } else {
        let message = format!("cd: {}: No such file or directory", arg);
        output.push(CmdOutput {output: message, is_err: true });
    }
}

fn try_execute_command(output: &mut Vec<CmdOutput>, cmd: &str, args: &[&str]) {
    if let Some(_path) = find_executable_in_path(cmd) {
        if let Ok(exe_output) = process::Command::new(cmd).args(args).output() {
            match str::from_utf8(&exe_output.stdout) {
                Ok(val) => {
                    let message = val.trim().to_string();
                    if !message.is_empty() {
                        output.push(CmdOutput {output: message, is_err: false })
                    }
                },
                Err(_) => output.push(CmdOutput {output: String::from("failed to parse program output as text"), is_err: true }),
            }
            match str::from_utf8(&exe_output.stderr) {
                Ok(val) => {
                    let message = val.trim().to_string();
                    if !message.is_empty() {
                        output.push(CmdOutput {output: message, is_err: true })
                    }
                },
                Err(_) => output.push(CmdOutput {output: String::from("failed to parse program output as text"), is_err: true }),
            }
        } else {
            let message = String::from("couldn't execute program");
            output.push(CmdOutput {output: message, is_err: true });
        }
    } else { 
        let message = format!("{}: command not found", cmd);
        output.push(CmdOutput {output: message, is_err: true });
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

fn write_output_to_file(output: &String, file_path: &String) {
    if let Ok(mut file) = File::create(file_path) {
        file.write(output.as_bytes()).expect("failed to write to file");
    }
}
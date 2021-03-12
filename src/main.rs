use clap::{load_yaml, App};
use std::io::{self, Write};
use std::{
    fmt,
    process::{exit, Child, Command as ProcessCommand, Stdio},
};

enum Shell {
    None,
    Bash,
}

impl From<&str> for Shell {
    fn from(value: &str) -> Self {
        match value {
            "bash" => Self::Bash,
            "none" => Self::None,
            x => panic!(format!("'{}' is not a valid shell value", x)),
        }
    }
}

#[derive(Debug)]
enum ProgramError<'a> {
    EmptyCommand,
    SpawnError(Command<'a>, std::io::Error),
}

impl<'a> fmt::Display for ProgramError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProgramError::EmptyCommand => write!(f, "Cannot run empty command"),
            ProgramError::SpawnError(command, error) => {
                write!(f, "Error running \"{}\": {:?}", command, error)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Command<'a> {
    program: &'a str,
    args: Vec<&'a str>,
}

impl<'a> Command<'a> {
    fn from(feature: &'a str, shell: &'a Shell) -> Result<Command<'a>, ProgramError<'a>> {
        if let Shell::Bash = shell {
            Ok(Command {
                program: "bash",
                args: vec!["-c", feature],
            })
        } else {
            let parts: Vec<&str> = feature.split_whitespace().collect();
            if parts.is_empty() {
                Err(ProgramError::EmptyCommand)
            } else {
                Ok(Command {
                    program: parts[0],
                    args: parts[1..].into(),
                })
            }
        }
    }

    fn spawn(&self) -> Result<Child, ProgramError> {
        ProcessCommand::new(self.program)
            .args(&self.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ProgramError::SpawnError(self.clone(), e))
    }
}

impl<'a> fmt::Display for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.program, self.args.join(" "))
    }
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let shell: String = matches.value_of("SHELL").unwrap_or("none").parse().unwrap();
    let shell = Shell::from(&shell[..]);

    let max_parallel = matches
        .value_of("MAX_PARALLEL")
        .unwrap_or("4")
        .parse()
        .unwrap();

    let mut commands: Vec<&str> = matches.values_of("INPUT").unwrap().collect();

    let mut children: Vec<Child> = Vec::with_capacity(max_parallel);

    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout_handle = stdout.lock();
    let mut stderr_handle = stderr.lock();

    let mut ok_exits: usize = 0;
    let mut err_exits: usize = 0;

    while !children.is_empty() || !commands.is_empty() {
        if children.len() != max_parallel && !commands.is_empty() {
            match Command::from(commands.remove(0), &shell) {
                Ok(command) => {
                    match command.spawn() {
                        Ok(child) => children.push(child),
                        Err(err) => {
                            err_exits += 1;
                            eprintln!("{}", err);
                        }
                    }
                    continue;
                }
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1);
                }
            }
        }

        let child = children.remove(0);
        let output = child.wait_with_output().unwrap();
        stdout_handle.write_all(&output.stdout).unwrap();
        stderr_handle.write_all(&output.stderr).unwrap();
        if output.status.success() {
            ok_exits += 1;
        } else {
            err_exits += 1;
        }
    }
    let is_ok = ok_exits > 0 || err_exits == 0;
    exit(if is_ok { 0 } else { 1 });
}

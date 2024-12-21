use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind, Write};
use std::path::Path;
use std::process;
use std::thread;

pub const CURRENT_PATH: &str = "./";

#[derive(Debug, Clone, Copy)]
pub enum ReportKind<'a> {
    Recoverable(&'a str),
    Fatal,
}

impl<'a> ReportKind<'a> {
    pub fn report(self, stderr: &mut io::Stderr, error: &Error) -> io::Result<()> {
        let msg = match self {
            Self::Recoverable(path) => {
                String::new()
                    + path
                    + ": 
                " + error.to_string().as_str()
                    + "\n"
            }

            Self::Fatal => error.to_string() + "\n",
        };

        let _ = stderr.write(msg.as_bytes())?;

        Ok(())
    }
}

pub fn read_dir<P>(path: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let current_dir = fs::read_dir(path)?;

    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    for dir_content in current_dir {
        let dir_content = match dir_content {
            Ok(dir_content) => dir_content,
            Err(error) => match error.kind() {
                ErrorKind::PermissionDenied => {
                    ReportKind::Recoverable(
                        "Failed 
                    to read directories",
                    )
                    .report(&mut stderr, &error)
                    .unwrap_or(());
                    continue;
                }

                _ => {
                    ReportKind::Fatal.report(&mut stderr, &error).unwrap_or(());
                    continue;
                }
            },
        };

        let path = dir_content.path();

        let properties = match dir_content.metadata() {
            Ok(properties) => properties,
            Err(error) => match error.kind() {
                ErrorKind::PermissionDenied => {
                    ReportKind::Recoverable(path.to_str().unwrap_or(""))
                        .report(&mut stderr, &error)
                        .unwrap_or(());
                    continue;
                }

                _ => {
                    ReportKind::Fatal.report(&mut stderr, &error).unwrap_or(());
                    continue;
                }
            },
        };

        if properties.is_file() || properties.is_symlink() {
            let as_path_str = path.to_str().unwrap_or("").to_string() + "\n";
            let _ = stdout.write(as_path_str.as_bytes())?;

            continue;
        }

        if properties.is_dir() {
            let task = thread::spawn(move || read_dir(path));

            match task.join() {
                Ok(value) => value.unwrap_or(()),
                _ => continue,
            }
        }
    }

    Ok(())
}

fn main() {
    let mut stderr = io::stderr();

    let args: Vec<String> = env::args()
        .enumerate()
        .filter(|(n_args, _)| *n_args != 0 && *n_args != usize::MAX)
        .map(|(_, string)| string)
        .collect();

    if args.len() == 0 {
        match read_dir(CURRENT_PATH) {
            Ok(_) => return,
            Err(error) => {
                ReportKind::Fatal.report(&mut stderr, &error).unwrap_or(());
                process::exit(1);
            }
        }
    }

    for path in args {
        match read_dir(path) {
            Ok(_) => {}
            Err(error) => ReportKind::Fatal.report(&mut stderr, &error).unwrap_or(()),
        }
    }
}

use std::env;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::process;

pub const CURRENT_PATH: &str = "./";

#[derive(Debug, Clone, Copy)]
pub enum ReportKind<'a> {
    Recoverable(&'a str),
    Fatal,
}

impl<'a> ReportKind<'a> {
    pub fn report<B, M>(self, buffer: &mut B, msg: &M) -> io::Result<()>
    where
        B: Write,
        M: ToString,
    {
        let msg = match self {
            Self::Recoverable(path) => {
                String::new() + path + ": " + msg.to_string().as_str() + "\n"
            }

            Self::Fatal => msg.to_string() + "\n",
        };

        let _ = buffer.write(msg.as_bytes())?;

        Ok(())
    }
}

pub fn read_dir<P>(stdout: &mut io::Stdout, stderr: &mut io::Stderr, path: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let current_dir = fs::read_dir(path)?;

    for dir_content in current_dir {
        let dir_content = match dir_content {
            Ok(dir_content) => dir_content,

            Err(error) => match error.kind() {
                ErrorKind::PermissionDenied => {
                    ReportKind::Recoverable("Failed to read directories")
                        .report(stderr, &error)
                        .unwrap_or(());

                    continue;
                }

                _ => {
                    ReportKind::Fatal.report(stderr, &error).unwrap_or(());

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
                        .report(stderr, &error)
                        .unwrap_or(());

                    continue;
                }

                _ => {
                    ReportKind::Fatal.report(stderr, &error).unwrap_or(());

                    continue;
                }
            },
        };

        if properties.is_file() || properties.is_symlink() {
            let as_path_str = path.to_str().unwrap_or("").to_string() + "\n";
            let _ = stdout.lock().write(as_path_str.as_bytes())?;

            continue;
        }

        if properties.is_dir() {
            read_dir(stdout, stderr, path).unwrap_or(());
        }
    }

    Ok(())
}

fn main() {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mut has_error = false;
    let mut exit_code: i32 = 0;

    let args: Vec<String> = env::args()
        .enumerate()
        .filter(|(idx, _)| *idx != 0 && *idx != usize::MAX)
        .map(|(_, args)| args)
        .collect();

    if args.len() == 0 {
        match read_dir(&mut stdout, &mut stderr, CURRENT_PATH) {
            Ok(_) => return,
            Err(error) => {
                ReportKind::Fatal.report(&mut stderr, &error).unwrap_or(());
                process::exit(error.raw_os_error().unwrap_or(1));
            }
        }
    }

    for path in args {
        match read_dir(&mut stdout, &mut stderr, path) {
            Ok(_) => {}
            Err(error) => {
                ReportKind::Fatal.report(&mut stderr, &error).unwrap_or(());

                exit_code = error.raw_os_error().unwrap_or(1);
                has_error = true;
            }
        }
    }

    if has_error {
        process::exit(exit_code);
    }
}

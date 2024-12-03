use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::Path;
use std::process::ExitCode;

const CURRENT_PATH: &str = "./";

fn read_recursive<P>(path: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut current_dir = fs::read_dir(path)?;

    while let Some(dir_content) = current_dir.next() {
        let dir_content = dir_content?;
        let dir_prop = dir_content.metadata()?;

        if dir_prop.is_file() || dir_prop.is_symlink() {
            let as_path_str = dir_content.path();

            let as_path_str = as_path_str.to_str().ok_or_else(|| {
                let error_type = ErrorKind::InvalidData;
                let error_message = "Path did not contains valid unicode character";

                return Error::new(error_type, error_message);
            })?;

            println!("{}", as_path_str);
        }

        if dir_prop.is_dir() {
            read_recursive(dir_content.path())?;
        }
    }

    Ok(())
}

fn error(err: Error) -> ExitCode {
    eprintln!("{err}");

    return ExitCode::FAILURE;
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args()
        .enumerate()
        .filter(|(n_args, _)| *n_args != 0)
        .map(|(_, string)| string)
        .collect();

    if args.len() == 0 {
        match read_recursive(CURRENT_PATH) {
            Ok(..) => return ExitCode::SUCCESS,
            Err(err) => return error(err),
        }
    }

    for path in args {
        match read_recursive(path) {
            Ok(..) => {}
            Err(err) => return error(err),
        }
    }

    ExitCode::SUCCESS
}

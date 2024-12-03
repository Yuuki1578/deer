use std::env;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::Path;

const CURRENT_PATH: &str = "./";

fn read_recursive_inner<P>(path: P) -> io::Result<()>
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
            read_recursive_inner(dir_content.path())?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args()
        .enumerate()
        .filter(|(n_args, _)| *n_args != 0)
        .map(|(_, string)| string)
        .collect();

    if args.len() == 0 {
        return read_recursive_inner(CURRENT_PATH);
    }

    for path in args {
        read_recursive_inner(path)?;
    }

    Ok(())
}

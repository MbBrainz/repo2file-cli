mod default_ignore;

use default_ignore::DefaultIgnore;
use git2::Repository;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tempdir::TempDir;

#[derive(StructOpt)]
#[structopt(
    name = "repo2file",
    about = "Turn a code repository into a single text file."
)]
struct Cli {
    /// The directory or Git URL of the repository
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Files to ignore, separated by commas
    #[structopt(long, use_delimiter = true)]
    ignore_files: Option<Vec<String>>,

    /// Directories to ignore, separated by commas
    #[structopt(long, use_delimiter = true)]
    ignore_dirs: Option<Vec<String>>,

    /// Files to include, separated by commas (exclusive with --ignore-files and --ignore-dirs)
    #[structopt(long, use_delimiter = true, conflicts_with_all = &["ignore_files", "ignore_dirs"])]
    include_files: Option<Vec<String>>,

    /// Output file
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    /// Boolean to save error log, default is false
    #[structopt(short, long)]
    error_log: bool,
}

fn main() -> io::Result<()> {
    let args = Cli::from_args();

    let input_path = if is_github_url(&args.input) {
        let temp_dir = clone_repo_to_temp(&args.input).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to clone repository: {}", e),
            )
        })?;
        temp_dir.path().to_owned()
    } else {
        args.input.clone()
    };

    let output_path = args.output.clone().unwrap_or_else(|| {
        let current_dir = std::env::current_dir().unwrap();
        current_dir.join(current_dir.file_name().unwrap())
    });

    let mut output_file = File::create(output_path.clone().with_extension(".txt"))?;

    let mut error_log_file = if args.error_log {
        Some(File::create(output_path.with_extension(".error.log"))?)
    } else {
        None
    };

    for entry in WalkBuilder::new(input_path)
        .add_custom_ignore_filename(".ignore")
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
    {
        let path = entry.path();
        if should_include(path, &args, &DefaultIgnore::default()) {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    writeln!(
                        output_file,
                        "\n\n// File: {}\n\n{}",
                        path.display(),
                        content
                    )?;
                }
                Err(e) => {
                    write_error_to_log(&mut error_log_file, path, e)?;
                    continue;
                }
            }
        }
    }

    Ok(())
}

fn write_error_to_log(
    error_log_file: &mut Option<File>,
    path: &Path,
    e: io::Error,
) -> Result<(), io::Error> {
    Ok(if error_log_file.is_some() {
        writeln!(
            error_log_file.as_mut().unwrap(),
            "Error reading file {}: {}",
            path.display(),
            e
        )?;
    })
}

// Function to determine if a file should be included based on the arguments
fn should_include(path: &Path, args: &Cli, config: &DefaultIgnore) -> bool {
    let mut ignore_files: Vec<&str> = config.ignore_files.iter().map(String::as_str).collect();
    let mut ignore_dirs: Vec<&str> = config.ignore_dirs.iter().map(String::as_str).collect();

    if let Some(user_ignore_files) = &args.ignore_files {
        ignore_files.extend(user_ignore_files.iter().map(String::as_str));
    }

    if let Some(user_ignore_dirs) = &args.ignore_dirs {
        ignore_dirs.extend(user_ignore_dirs.iter().map(String::as_str));
    }

    let mut glob_builder = GlobSetBuilder::new();
    for pattern in &ignore_files {
        glob_builder.add(Glob::new(pattern).unwrap());
    }
    let glob_set = glob_builder.build().unwrap();

    if let Some(include_files) = &args.include_files {
        return include_files.iter().any(|f| path.ends_with(f));
    }

    let path_str = path.to_str().unwrap_or_default();

    if glob_set.is_match(path_str) || ignore_files.iter().any(|&f| path.ends_with(f)) {
        return false;
    }

    if ignore_dirs
        .iter()
        .any(|&d| path.components().any(|comp| comp.as_os_str() == d))
    {
        return false;
    }

    true
}

fn is_github_url(path: &Path) -> bool {
    path.to_str()
        .map_or(false, |s| s.starts_with("https://github.com/"))
}

fn clone_repo_to_temp(url: &Path) -> Result<TempDir, io::Error> {
    let temp_dir = TempDir::new("temp-repo2file")?;
    Repository::clone(url.to_str().unwrap(), temp_dir.path()).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to clone repository: {}", e),
        )
    })?;
    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_ignore_files() -> DefaultIgnore {
        return DefaultIgnore {
            ignore_files: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".vscode".to_string(),
                "*.lock".to_string(),
            ],
            ignore_dirs: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".vscode".to_string(),
            ],
        };
    }

    #[test]
    fn test_should_include_no_ignore_no_include() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: None,
            include_files: None,
            output: Some(PathBuf::from("output.txt")),
            error_log: false,
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(should_include(&path, &args, &default_ignore_files()));
        let lock_path = PathBuf::from("input/Cargo.lock");
        assert!(!should_include(&lock_path, &args, &default_ignore_files()));
    }

    #[test]
    fn test_should_include_with_ignore_files() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: Some(vec!["test_file.txt".to_string()]),
            ignore_dirs: None,
            include_files: None,
            output: Some(PathBuf::from("output.txt")),
            error_log: false,
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(!should_include(&path, &args, &default_ignore_files()));

        let other_path = PathBuf::from("input/other_file.txt");
        assert!(should_include(&other_path, &args, &default_ignore_files()));
    }

    #[test]
    fn test_should_include_with_ignore_dirs() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: Some(vec!["ignore_dir".to_string()]),
            include_files: None,
            output: Some(PathBuf::from("output.txt")),
            error_log: false,
        };

        let path = PathBuf::from("input/ignore_dir/test_file.txt");
        assert!(!should_include(&path, &args, &default_ignore_files()));

        let other_path = PathBuf::from("input/other_dir/test_file.txt");
        assert!(should_include(&other_path, &args, &default_ignore_files()));
    }

    #[test]
    fn test_should_include_with_include_files() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: None,
            include_files: Some(vec!["include_file.txt".to_string()]),
            output: Some(PathBuf::from("output.txt")),
            error_log: false,
        };

        let path = PathBuf::from("input/include_file.txt");
        assert!(should_include(&path, &args, &default_ignore_files()));

        let other_path = PathBuf::from("input/other_file.txt");
        assert!(!should_include(&other_path, &args, &default_ignore_files()));
    }

    #[test]
    fn test_should_include_with_ignore_and_include() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: Some(vec!["test_file.txt".to_string()]),
            ignore_dirs: Some(vec!["ignore_dir".to_string()]),
            include_files: None,
            output: Some(PathBuf::from("output.txt")),
            error_log: false,
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(!should_include(&path, &args, &default_ignore_files()));

        let dir_path = PathBuf::from("input/ignore_dir/test_file.txt");
        assert!(!should_include(&dir_path, &args, &default_ignore_files()));

        let other_path = PathBuf::from("input/other_file.txt");
        assert!(should_include(&other_path, &args, &default_ignore_files()));
    }

    #[test]
    fn test_should_include_with_multiple_ignore_files() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: Some(vec![
                "test_file.txt".to_string(),
                "ignore_file.txt".to_string(),
            ]),
            ignore_dirs: None,
            include_files: None,
            output: Some(PathBuf::from("output.txt")),
            error_log: false,
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(!should_include(&path, &args, &default_ignore_files()));

        let path = PathBuf::from("input/ignore_file.txt");
        assert!(!should_include(&path, &args, &default_ignore_files()));

        let path = PathBuf::from("input/valid_file.txt");
        assert!(should_include(&path, &args, &default_ignore_files()));
    }

    #[test]
    fn test_should_include_with_multiple_ignore_dirs() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: Some(vec!["ignore_dir1".to_string(), "ignore_dir2".to_string()]),
            include_files: None,
            output: Some(PathBuf::from("output.txt")),
            error_log: false,
        };

        let path1 = PathBuf::from("input/ignore_dir1/test_file.txt");
        assert!(!should_include(&path1, &args, &default_ignore_files()));

        let path2 = PathBuf::from("input/ignore_dir2/test_file.txt");
        assert!(!should_include(&path2, &args, &default_ignore_files()));

        let valid_path = PathBuf::from("input/valid_dir/test_file.txt");
        assert!(should_include(&valid_path, &args, &default_ignore_files()));
    }

    #[test]
    fn test_is_github_url() {
        assert!(is_github_url(&PathBuf::from(
            "https://github.com/username/repo"
        )));
        assert!(!is_github_url(&PathBuf::from(
            "http://github.com/username/repo"
        )));
        assert!(!is_github_url(&PathBuf::from(
            "https://gitlab.com/username/repo"
        )));
        assert!(!is_github_url(&PathBuf::from("/local/path/to/repo")));
    }
}

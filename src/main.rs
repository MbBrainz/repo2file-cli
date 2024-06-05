use git2::Repository;
use ignore::WalkBuilder;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tempdir::TempDir;

#[derive(StructOpt)]
#[structopt(
    name = "repo2file-cli",
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
    output: PathBuf,
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

    let mut output_file = File::create(&args.output)?;

    for entry in WalkBuilder::new(input_path)
        .add_custom_ignore_filename(".ignore")
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
    {
        let path = entry.path();
        if should_include(path, &args) {
            let content = std::fs::read_to_string(path)?;
            writeln!(
                output_file,
                "\n\n// File: {}\n\n{}",
                path.display(),
                content
            )?;
        }
    }

    Ok(())
}

// Function to determine if a file should be included based on the arguments
fn should_include(path: &Path, args: &Cli) -> bool {
    // Define default ignore files and directories
    let default_ignore_files = vec!["node_modules", "target", ".vscode"];
    let default_ignore_dirs = vec!["node_modules", "target", ".vscode"];

    // Merge user-provided and default ignore files
    let ignore_files: Vec<&str> =
        args.ignore_files
            .as_ref()
            .map_or(default_ignore_files.clone(), |v| {
                v.iter()
                    .map(String::as_str)
                    .chain(default_ignore_files.iter().copied())
                    .collect()
            });

    // Merge user-provided and default ignore directories
    let ignore_dirs: Vec<&str> =
        args.ignore_dirs
            .as_ref()
            .map_or(default_ignore_dirs.clone(), |v| {
                v.iter()
                    .map(String::as_str)
                    .chain(default_ignore_dirs.iter().copied())
                    .collect()
            });

    // If include_files is specified, only include those files
    if let Some(include_files) = &args.include_files {
        return include_files.iter().any(|f| path.ends_with(f));
    }

    // Check if the file should be ignored based on ignore_files
    if ignore_files.iter().any(|&f| path.ends_with(f)) {
        return false;
    }

    // Check if the file is in a directory that should be ignored
    if ignore_dirs
        .iter()
        .any(|&d| path.components().any(|comp| comp.as_os_str() == d))
    {
        return false;
    }

    true // Include the file by default if no exclusion criteria match
}

fn is_github_url(path: &Path) -> bool {
    path.to_str()
        .map_or(false, |s| s.starts_with("https://github.com/"))
}

fn clone_repo_to_temp(url: &Path) -> Result<TempDir, git2::Error> {
    let temp_dir = TempDir::new("temp-repo2file").unwrap();
    Repository::clone(url.to_str().unwrap(), temp_dir.path())?;
    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_include_no_ignore_no_include() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: None,
            include_files: None,
            output: PathBuf::from("output.txt"),
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(should_include(&path, &args));
    }

    #[test]
    fn test_should_include_with_ignore_files() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: Some(vec!["test_file.txt".to_string()]),
            ignore_dirs: None,
            include_files: None,
            output: PathBuf::from("output.txt"),
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(!should_include(&path, &args));

        let other_path = PathBuf::from("input/other_file.txt");
        assert!(should_include(&other_path, &args));
    }

    #[test]
    fn test_should_include_with_ignore_dirs() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: Some(vec!["ignore_dir".to_string()]),
            include_files: None,
            output: PathBuf::from("output.txt"),
        };

        let path = PathBuf::from("input/ignore_dir/test_file.txt");
        assert!(!should_include(&path, &args));

        let other_path = PathBuf::from("input/other_dir/test_file.txt");
        assert!(should_include(&other_path, &args));
    }

    #[test]
    fn test_should_include_with_include_files() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: None,
            include_files: Some(vec!["include_file.txt".to_string()]),
            output: PathBuf::from("output.txt"),
        };

        let path = PathBuf::from("input/include_file.txt");
        assert!(should_include(&path, &args));

        let other_path = PathBuf::from("input/other_file.txt");
        assert!(!should_include(&other_path, &args));
    }

    #[test]
    fn test_should_include_with_ignore_and_include() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: Some(vec!["test_file.txt".to_string()]),
            ignore_dirs: Some(vec!["ignore_dir".to_string()]),
            include_files: None,
            output: PathBuf::from("output.txt"),
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(!should_include(&path, &args));

        let dir_path = PathBuf::from("input/ignore_dir/test_file.txt");
        assert!(!should_include(&dir_path, &args));

        let other_path = PathBuf::from("input/other_file.txt");
        assert!(should_include(&other_path, &args));
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
            output: PathBuf::from("output.txt"),
        };

        let path = PathBuf::from("input/test_file.txt");
        assert!(!should_include(&path, &args));

        let other_path = PathBuf::from("input/ignore_file.txt");
        assert!(!should_include(&other_path, &args));

        let valid_path = PathBuf::from("input/valid_file.txt");
        assert!(should_include(&valid_path, &args));
    }

    #[test]
    fn test_should_include_with_multiple_ignore_dirs() {
        let args = Cli {
            input: PathBuf::from("input"),
            ignore_files: None,
            ignore_dirs: Some(vec!["ignore_dir1".to_string(), "ignore_dir2".to_string()]),
            include_files: None,
            output: PathBuf::from("output.txt"),
        };

        let path1 = PathBuf::from("input/ignore_dir1/test_file.txt");
        assert!(!should_include(&path1, &args));

        let path2 = PathBuf::from("input/ignore_dir2/test_file.txt");
        assert!(!should_include(&path2, &args));

        let valid_path = PathBuf::from("input/valid_dir/test_file.txt");
        assert!(should_include(&valid_path, &args));
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

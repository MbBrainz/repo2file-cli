use serde::Deserialize;

#[derive(Deserialize)]
pub struct DefaultIgnore {
    pub ignore_files: Vec<String>,
    pub ignore_dirs: Vec<String>,
}

impl Default for DefaultIgnore {
    fn default() -> Self {
        DefaultIgnore {
            ignore_dirs: ["node_modules", ".git", ".idea", ".vscode"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ignore_files: IntoIterator::into_iter([
                "*LICENCE.md",
                "*CHANGELOG.md",
                "*.DS_Store",
                "*.all-contributorsrc",
                "*.yaml",
                "*.yml",
                "*.json",
                "*.csv",
                "*.svg",
                "*.conf",
                "*.ini",
                "*.env",
                "*.log",
                "*.tmp",
                "*.pyc",
                "*.class",
                "*.o",
                "*.obj",
                "*.exe",
                "*.dll",
                "*.so",
                "*.dylib",
                "*.ncb",
                "*.sdf",
                "*.suo",
                "*.pdb",
                "*.idb",
                "*.lock",
                "*.toml",
                ".prettierrc.*",
                "*.txt",
                "Pipfile",
                "*.cfg",
                ".gitignore",
                ".gitattributes",
                ".dockerignore",
                ".env",
                ".flaskenv",
                ".editorconfig",
                "Makefile",
                "CMakeLists.txt",
            ])
            .map(|s| s.to_string())
            .collect(),
        }
    }
}

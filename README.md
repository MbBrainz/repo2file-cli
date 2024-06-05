# repo2file-cli

`repo2file-cli` is a command-line tool designed to consolidate a code repository into a single text file. This can be useful for archiving, analysis, or sharing purposes.

## Installation

To install `repo2file-cli`, you need to have Rust installed. If you don't have Rust installed, you can get it from [rustup.rs](https://rustup.rs/).

Once you have Rust, you can install the `repo2file-cli` using `cargo`:

```sh
cargo install repo2file-cli
```

## Usage

To use `repo2file-cli`, you can run the following command:

```sh
repo2file-cli <input> [OPTIONS]
```

### Arguments

- `<input>`: The directory or Git URL of the repository you want to process.

### Options

- `--ignore-files <FILES>`: Comma-separated list of files to ignore.
- `--ignore-dirs <DIRS>`: Comma-separated list of directories to ignore.
- `--include-files <FILES>`: Comma-separated list of files to include exclusively (cannot be used with `--ignore-files` or `--ignore-dirs`).
- `--output <OUTPUT>`: The output file. Defaults to a file named after the current directory.

### Examples

#### Convert a local repository

```sh
repo2file-cli /path/to/repository --output output.txt
```

#### Convert a GitHub repository

```sh
repo2file-cli https://github.com/username/repo --output output.txt
```

#### Ignore specific files and directories

```sh
repo2file-cli /path/to/repository --ignore-files *.md,*.json --ignore-dirs node_modules,.git
```

#### Include only specific files

```sh
repo2file-cli /path/to/repository --include-files *.rs,*.toml
```

## Contributing

We welcome contributions! Please follow these steps to contribute:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Make your changes.
4. Commit your changes (`git commit -am 'Add new feature'`).
5. Push to the branch (`git push origin feature-branch`).
6. Create a new Pull Request.

### Development Setup

To set up your development environment, follow these steps:

1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/repo2file-cli.git
    ```
2. Change to the project directory:
    ```sh
    cd repo2file-cli
    ```
3. Install the required extensions (if using VSCode):
    - [Rust (rls)](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust)
    - [Crates](https://marketplace.visualstudio.com/items?itemName=serayuzgur.crates)
    - [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)

### Running Tests

You can run the tests using the following command:

```sh
cargo test
```
### Build from source

you can install the binary from source while youre devin'


```sh
cargo install --path .
```



## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE.md) file for details.
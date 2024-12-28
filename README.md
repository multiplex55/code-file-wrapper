# Code File Wrapper

`Code File Wrapper` is a Rust CLI application that creates a text file (`tags_output.txt`) containing the contents of human-readable files (based on certain file extensions) in a selected or provided directory, wrapped in tags.

## Overview

This application either:
1. Uses an **optional command-line argument** (a full path to a directory).
2. Or, if no path is provided, launches a **folder-picker dialog** for you to select a directory.

All files in that directory with the following extensions are considered human-readable and processed:
- `.ini`
- `.txt`
- `.rs`
- `.cs`
- `.json`
- `.xml`

Any files with extensions not included in this list (e.g., `.exe`, `.pdb`) are ignored.

## Usage

1. **Build** the project in the root directory:
   ```bash
   cargo build
   ```
2. **Run** the binary:
   ```bash
   cargo run -- [optional_path]
   ```
   - If you **do not** provide a path, a **folder picker** will open.
   - If you **provide a path** (e.g., `C:\my_project` on Windows or `/home/user/my_project` on Linux), the application will immediately process that path without opening the folder picker.
   - If the provided path does not exist or is invalid, the application will exit.

## Output

The application creates an output file named `tags_output.txt` in the **current working directory**. This file contains entries in the following format:

```plaintext
<filename.ext>
contents of filename.ext
</filename.ext>
```

Each file in the processed folder that meets the extension criteria will have its contents inserted into the `tags_output.txt` file, wrapped in opening and closing tags corresponding to the file name.

### Example

Suppose the directory `my_rust_project/src` has the following files:
- `main.rs`
- `utils.rs`
- `notes.txt`
- `logo.png`  _(not processed, since `.png` is not in the valid list)_

After running:

```bash
cargo run -- /path/to/my_rust_project/src
```

The resulting `tags_output.txt` would contain something like:

```
<main.rs>
// contents of main.rs
</main.rs>

<utils.rs>
// contents of utils.rs
</utils.rs>

<notes.txt>
# contents of notes.txt
</notes.txt>
```

## Supported Platforms

- macOS
- Linux
- Windows

### Dependencies

- [Rust](https://www.rust-lang.org/) (1.60+ recommended)
- [Clap 4.3+](https://crates.io/crates/clap)
- [rfd 0.12+](https://crates.io/crates/rfd)

## License

This project is distributed under the terms of the [MIT license](LICENSE).

## Contributing

Contributions, suggestions, and improvements are welcome! Feel free to open an issue or create a pull request.
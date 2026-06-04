# 🗂️ **Code File Wrapper**

<p align="center">
  <b>Wrap your code files inside XML-style tags effortlessly! 🚀</b> <br>
  <i>Includes GUI mode, clipboard support, recursion, preset commands, and more!</i>
</p>

<p align="center">
  <img alt="GitHub repo size" src="https://img.shields.io/github/repo-size/multiplex55/code-file-wrapper">
  <img alt="GitHub stars" src="https://img.shields.io/github/stars/multiplex55/code-file-wrapper?style=social">
  <img alt="GitHub license" src="https://img.shields.io/github/license/multiplex55/code-file-wrapper">
  <img alt="Rust Version" src="https://img.shields.io/badge/Rust-Edition%202021-orange">
</p>

---

## 📌 Version  
- **Latest Release:** `1.1.0`  
- **Author:** `Multiplex55`  
- **License:** MIT  
- **Platform:** 🖥 Windows and Linux

---

## ✨ Features  

✅ Wraps code files in clean XML-style tags based on relative paths  
✅ File extension filtering (`.rs`, `.json`, `.lua`, `.ahk`, etc.)  
✅ 🔧 GUI-based File Type Group Manager (create/edit/delete extension sets)  
✅ Modern GUI with native folder picker and filetype selector  
✅ Recursive folder traversal (optional)  
✅ Folder exclusion rules (e.g., `.git`, `target`)  
✅ Additional command section + preset text insertion  
✅ Clipboard copy support (UTF-16 Windows clipboard)  
✅ Preset manager to create/edit reusable instruction blocks  
✅ Uses `eframe` + `egui` for fast native GUI  
✅ Optimized for AI prompt pipelines and full-project context packing  

---

## 📥 Installation

### Prerequisites  
- 🦀 [Rust](https://www.rust-lang.org/tools/install) installed

### Build From Source  
```sh
git clone https://github.com/multiplex55/code-file-wrapper.git
cd code-file-wrapper
cargo build --release
```

---

## 📦 Dependencies

| Crate           | Purpose                                      |
|----------------|----------------------------------------------|
| `eframe`        | Native GUI framework                        |
| `rfd`           | Native file/folder picker                   |
| `clipboard-win` | Clipboard access (Windows-only)             |
| `windows`       | Win32 bindings (`GetCursorPos`, etc.)       |
| `serde`         | JSON serialization for preset + filetypes   |

---

## 🚀 Usage

### Launch GUI  
```sh
./target/release/code-file-wrapper
```

### GUI Actions  
- Select directory containing source files  
- Choose or manage filetype groups (Rust, JSON, Web, etc.)  
- Enable recursion and ignore folders if needed  
- Add manual instructions or select a preset  
- Output is written to `tags_output.txt` by default, or to the custom path you enter  
- Optionally copies output to your clipboard  

### CLI Examples

The CLI uses the same shared generation engine as the GUI, so tagged output, recursive traversal, ignored folders, additional commands, presets, and clipboard copy behavior are generated consistently whether you run interactively or from a script.

Run without a subcommand to launch the GUI:

```sh
code-file-wrapper
```

Launch the GUI explicitly:

```sh
code-file-wrapper gui
```

List configured file type groups from `filetypes.json`:

```sh
code-file-wrapper list-file-types
```

Generate Rust context from the current directory recursively. Without `--output`, the CLI writes to the default `tags_output.txt`:

```sh
code-file-wrapper run --dir . --file-type Rust --recursive
```

Use a custom output filename for repeatable no-GUI workflows:

```sh
code-file-wrapper run --dir . --file-type Rust --recursive --output rust_context.txt
```

Select individual extensions instead of a file type group. Repeat `--ext` once per extension:

```sh
code-file-wrapper run --dir . --ext rs --ext toml --ext md --recursive
```

Copy the generated output to the clipboard after writing the output file:

```sh
code-file-wrapper run --dir . --file-type Rust --recursive --copy
```

Profiles are optional convenience helpers for saving command arguments, but they are not required for repeatability. A checked-in shell, PowerShell, or batch script that calls `code-file-wrapper run` with explicit arguments is fully repeatable without using profiles.

#### PowerShell Script Template

```powershell
$CodeFileWrapper = "C:\Tools\code-file-wrapper\code-file-wrapper.exe"
$ProjectPath = "C:\Projects\my-rust-app"
$OutputPath = Join-Path $ProjectPath "rust_context.txt"
$CopyToClipboard = $false  # Set to $true to also copy output after writing the file.

$Arguments = @(
    "run",
    "--dir", $ProjectPath,
    "--file-type", "Rust",
    "--recursive",
    "--ignore", "target",
    "--ignore", ".git",
    "--output", $OutputPath
)

if ($CopyToClipboard) {
    $Arguments += "--copy"
}

& $CodeFileWrapper @Arguments
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
```

#### Windows Batch Script Template

```bat
@echo off
set "CODE_FILE_WRAPPER=C:\Tools\code-file-wrapper\code-file-wrapper.exe"
set "PROJECT_PATH=C:\Projects\my-rust-app"
set "OUTPUT_PATH=%PROJECT_PATH%\rust_context.txt"
set "COPY_TO_CLIPBOARD=0"

set "COPY_ARG="
if "%COPY_TO_CLIPBOARD%"=="1" set "COPY_ARG=--copy"

"%CODE_FILE_WRAPPER%" run ^
  --dir "%PROJECT_PATH%" ^
  --file-type Rust ^
  --recursive ^
  --ignore target ^
  --ignore .git ^
  --output "%OUTPUT_PATH%" ^
  %COPY_ARG%

if errorlevel 1 exit /b %errorlevel%
```

### Windows CLI and GUI Tradeoff

The Windows build uses the normal console subsystem so CLI output and errors are visible in PowerShell. This means commands such as `code-file-wrapper list-file-types` print their results normally, and invalid CLI invocations show errors with non-zero exit codes.

The tradeoff is that launching the GUI build directly on Windows may show a console window alongside the graphical interface. A future two-binary design could provide `code-file-wrapper.exe` for CLI use and `code-file-wrapper-gui.exe` with `windows_subsystem = "windows"` for GUI-only launch without a console window.

---

## 📜 How It Works

```mermaid
graph TD;
    A[Start] --> B[Open GUI]
    B --> C[Select Directory & File Type Group]
    C --> D[Enable Recursive Search?]
    D -->|Yes| E[Setup Ignore Folder List]
    D -->|No| F[Proceed]
    E --> G[Scan Files]
    F --> G
    G --> H[Wrap Files with <RelativePath> Tags]
    H --> I[Write to selected output file]
    I --> J[Append Presets + Additional Commands]
    J --> K{Copy to Clipboard?}
    K -->|Yes| L[Copy Output]
    K -->|No| M[Prompt to Open File]
    L --> N[Done]
    M --> N
```

---

## 📄 Output Example

```xml
<src/lib.rs>
pub fn greet() {
    println!("Hello!");
}
</src/lib.rs>

[Additional Commands]
TODO: Review error handling in all entry points.
```

---

## 📚 Use Cases

### 🧠 AI Prompt Construction  
Bundle multiple files into one tagged blob, ready to feed into ChatGPT, Claude, etc.

### 📎 Support / Bug Reports  
Instantly paste full project context into GitHub issues or support threads.

### 🛠️ Offline Processing  
Prepare files for static analysis, formatting, or documentation via downstream tools.

---

## 🔧 Preset System

Define common instruction blocks once and reuse them via dropdown.  
Includes examples like:

- "Create Function Documentation"  
- "Create Readme"  
- Custom slots (`Button 3–5`) for expansion

Presets are stored in `presets.json` and can be fully managed via the GUI.

---

## 🧩 File Type Groups

Manage which file extensions to include via editable file groups.  
Stored in `filetypes.json`, managed in GUI under **Manage File Types**.

Examples:
- **Rust** → `["rs"]`  
- **Web** → `["html", "css", "js"]`  
- **JSON & Config** → `["json", "ron", "toml"]`

---

## 🤝 Contributing

🎉 All contributions are welcome!  

1. Fork this repo  
2. Create a new branch  
3. Commit your changes  
4. Submit a pull request 🚀

---

## ⚖️ License

MIT License — free to use, modify, and distribute.  

---

## 🌟 Like the Project?

If this saved you time or helped your workflow,  
**please star the repo!** ⭐

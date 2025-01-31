<h1 align="center">Code File Wrapper</h1>

<p align="center">
  <b>Wrap your code files inside XML-style tags effortlessly! 🚀</b> <br>
  <i>Supports multiple languages, GUI mode, clipboard integration, and more!</i>
</p>

<p align="center">
  <img alt="GitHub repo size" src="https://img.shields.io/github/repo-size/multiplex55/code-file-wrapper">
  <img alt="GitHub stars" src="https://img.shields.io/github/stars/multiplex55/code-file-wrapper?style=social">
  <img alt="GitHub license" src="https://img.shields.io/github/license/multiplex55/code-file-wrapper">
  <img alt="Rust Version" src="https://img.shields.io/badge/Rust-Edition%202021-orange">
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.0-blue">
</p>

---

## 🏆 Author  
**Multiplex55**  

---

## ✨ Features
✅ **Wraps files inside XML-like tags**  
✅ **Supports multiple languages** (`.rs`, `.json`, `.xml`, `.c`, `.cpp`, `.ahk`, etc.)  
✅ **GUI Mode & CLI Mode** 🎨📜  
✅ **Clipboard Integration** 📋  
✅ **Multi-line Notes Section** 📝  
✅ **Error Handling & Debugging Messages** ⚠️  
✅ **Lightweight & Blazing Fast** ⚡  
✅ **Useful for providing context to AIs or documentation** 🤖  

---

## 📦 Dependencies

| Dependency  | Version  | Description  | Link  |
|------------|---------|-------------|-------|
| `clap` | `4.5.23` | Command-line argument parser with derive macros | [📌 Clap Crate](https://crates.io/crates/clap) |
| `clipboard` | `0.5.0` | Clipboard API for copying text | [📌 Clipboard Crate](https://crates.io/crates/clipboard) |
| `eframe` | `0.30.0` | GUI framework for Rust applications | [📌 eframe Crate](https://crates.io/crates/eframe) |
| `rfd` | `0.15.2` | File picker dialog for GUI | [📌 rfd Crate](https://crates.io/crates/rfd) |
| `windows` | `0.59.0` | Windows API bindings for Rust | [📌 Windows Crate](https://crates.io/crates/windows) |
| `winit` | `0.30.8` | Window handling for GUI applications | [📌 Winit Crate](https://crates.io/crates/winit) |

---

## 📥 Installation

**Prerequisites:**  
- 🦀 [Rust Installed](https://www.rust-lang.org/tools/install)  

Clone the repo and build:  

```sh
git clone https://github.com/multiplex55/code-file-wrapper.git
cd code-file-wrapper
cargo build --release
```

---

## 🚀 Usage  

### **CLI Mode**  

To run **CLI mode**, provide a folder path:  

```sh
./code-file-wrapper /path/to/directory
```

### **GUI Mode**  
To launch the interactive **GUI mode**, just run:

```sh
./code-file-wrapper
```

**GUI Features:**  
✔️ **Select a processing mode**  
✔️ **Enable automatic clipboard saving**  
✔️ **Provide additional multi-line notes**  

---

## 📜 How It Works  

### **Flowchart of Execution**  
```mermaid
graph TD;
    A[Start] -->|CLI Path Provided| B[Process Folder]
    A -->|No Path Provided| C[Open GUI]
    B --> D[Filter & Read Files]
    C --> E[User Selects Mode & Options]
    D --> F[Wrap Content in Tags]
    E --> F
    F --> G[Write to tags_output.txt]
    G --> H[Append User Notes]
    H -->|Clipboard Enabled| I[Copy to Clipboard]
    H -->|Exit| J[Done! 🎉]
```

---
## ➡️ Output
**Final Output Example (`tags_output.txt`):**  
```
<main.rs>
fn main() {
    println!("Hello, world!");
}
</main.rs>

[Additional Commands]
TODO: Add feature X...
```

---

## 🤝 Contributing  

🎉 **We welcome contributions!** 🎉  

To contribute:  
1. **Fork** the repository  
2. Create a **feature branch**  
3. **Submit a pull request**  

---

## ⚖️ License  

📜 **MIT License** - Feel free to modify and distribute!  

---

## 🌟 Show Some Love!  

If you like this project, **drop a ⭐ on GitHub!**  

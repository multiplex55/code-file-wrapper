# 🗂️ **Code File Wrapper**  

<p align="center">
  <b>Wrap your code files inside XML-style tags effortlessly! 🚀</b> <br>
  <i>Supports multiple languages, GUI mode, clipboard integration, folder ignoring, and more!</i>
</p>

<p align="center">
  <img alt="GitHub repo size" src="https://img.shields.io/github/repo-size/multiplex55/code-file-wrapper">
  <img alt="GitHub stars" src="https://img.shields.io/github/stars/multiplex55/code-file-wrapper?style=social">
  <img alt="GitHub license" src="https://img.shields.io/github/license/multiplex55/code-file-wrapper">
  <img alt="Rust Version" src="https://img.shields.io/badge/Rust-Edition%202021-orange">
</p>

---

## 📌 Version  
- **Latest Release:** `0.5.0`
- **Author:** `Multiplex55`
- **License:** MIT  
- **Platform:** 🖥 Windows Only  

---

## ✨ Features  
✅ **Wraps code files inside XML-like tags**  
✅ **Supports multiple file types** (`.rs`, `.json`, `.xml`, `.c`, `.cpp`, `.h`, `.lua`, `.ahk`)  
✅ **Modern Graphical User Interface (GUI)** 🎨  
✅ **Automatic Clipboard Copy Option** 📋  
✅ **Recursive Directory Search** 🔍  
✅ **Folder Ignoring Rules** 📂 (skip `target/`, `.git/`, etc.)  
✅ **Multi-line Additional Commands Section** 📝  
✅ **Preset Command Templates** ⚙️  
✅ **Error Handling and Warnings** ⚠️  
✅ **Fast and Lightweight** ⚡  
✅ **Optimized for AI Code Context or Documentation Workflows** 🤖  

---

## 📥 Installation  

### **Prerequisites:**  
- 🦀 [Rust Installed](https://www.rust-lang.org/tools/install)  

### **Build From Source:**  
```sh
git clone https://github.com/multiplex55/code-file-wrapper.git
cd code-file-wrapper
cargo build --release
```

---

## 📦 Dependencies  

| Name             | Version | Description                                  | Link |
|------------------|---------|----------------------------------------------|------|
| `eframe`         | `0.30.0` | GUI framework for building native apps      | [📦 Crates.io](https://crates.io/crates/eframe) |
| `rfd`            | `0.15.2` | Native file/folder dialog library           | [📦 Crates.io](https://crates.io/crates/rfd) |
| `clipboard-win`  | `4.4.1`  | Windows clipboard API binding               | [📦 Crates.io](https://crates.io/crates/clipboard-win) |
| `windows`        | `0.59.0` | Win32 API bindings for Rust                 | [📦 Crates.io](https://crates.io/crates/windows) |
| `winit`          | `0.30.8` | Window creation and event handling library | [📦 Crates.io](https://crates.io/crates/winit) |

---

## 🚀 Usage  

### **Launching the GUI Mode**  
After building, simply run the executable:

```sh
./code-file-wrapper
```

**Main GUI Options:**  
✔️ Select processing file mode (Rust, JSON, etc.)  
✔️ Choose a target directory  
✔️ Enable recursive search  
✔️ Ignore unwanted folders (optional)  
✔️ Add custom multi-line additional commands  
✔️ Select preset templates  
✔️ Copy output to clipboard automatically

---

## 📜 How It Works  

### **Updated Execution Flowchart**  
```mermaid
graph TD;
    A[Start] --> B[Open GUI]
    B --> C[User Selects Directory]
    C --> D[User Chooses File Type]
    D --> E[User Adds Additional Commands or Presets]
    E --> F[Enable Recursive Search?]
    F -->|Yes| G[Ignore Folders Setup]
    F -->|No| H[Skip Folder Ignore]
    G --> I[Process Matching Files Recursively]
    H --> I
    I --> J[Wrap Each File Content with XML-style Tags]
    J --> K[Write to tags_output.txt]
    K --> L[Append Additional Commands Section]
    L --> M{Clipboard Option Enabled?}
    M -->|Yes| N[Copy Output to Clipboard]
    M -->|No| O[Skip Clipboard Copy]
    N --> P[Success: Exit]
    O --> P
```

---

## ➡️ Output Example  

**Generated `tags_output.txt`:**
```xml
<main.rs>
fn main() {
    println!("Hello, world!");
}
</main.rs>

[Additional Commands]
TODO: Add refactoring checklist...
```

---

## 📊 Example Use Cases  

### 📝 **AI Context Provider**
- Perfect for giving ChatGPT, Claude, or other AIs a "full project view" in a single prompt.
- Wraps your files cleanly so AI models can understand structure better.

### 📚 **Automated Documentation Extraction**
- Capture important source files into portable text blobs.
- Pipe into documentation generators, git history comments, etc.

### 📋 **Clipboard Efficiency**
- Instantly copy processed output into GitHub issues, bug reports, or tech support emails.

---

## 🤝 Contributing  

🎉 **Contributions are welcome!** 🎉  

- Fork the repo  
- Create your own branch  
- Make your changes  
- Submit a pull request 🚀

---

## ⚖️ License  

📜 **MIT License**  
- Free to use, modify, and distribute!  

---

## 🌟 Show Some Love!  

If you found this project helpful, **drop a ⭐ on GitHub!**  

---

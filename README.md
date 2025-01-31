# 🗂️ **Code File Wrapper**  

<p align="center">
  <b>Wrap your code files inside XML-style tags effortlessly! 🚀</b> <br>
  <i>Supports multiple languages, GUI mode, clipboard integration, and more!</i>
</p>

<p align="center">
  <img alt="GitHub repo size" src="https://img.shields.io/github/repo-size/multiplex55/code-file-wrapper">
  <img alt="GitHub stars" src="https://img.shields.io/github/stars/multiplex55/code-file-wrapper?style=social">
  <img alt="GitHub license" src="https://img.shields.io/github/license/multiplex55/code-file-wrapper">
  <img alt="Rust Version" src="https://img.shields.io/badge/Rust-Edition%202021-orange">
</p>

---

## 📌 Version  
- **Latest Release:** `0.3.0`
- **Author:** `Multiplex55`
- **License:** MIT  
- **Platform:** 🖥 Windows  

---

## ✨ Features  
✅ **Wraps files inside XML-like tags**  
✅ **Supports multiple languages** (`.rs`, `.json`, `.xml`, `.c`, `.cpp`, `.ahk`, etc.)  
✅ **Graphical User Interface (GUI)** 🎨  
✅ **Clipboard Integration** 📋  
✅ **Multi-line Notes Section** 📝  
✅ **Error Handling & Debugging Messages** ⚠️  
✅ **Lightweight & Blazing Fast** ⚡  
✅ **Useful for providing context to AIs or documentation** 🤖  

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
| Name             | Version | Description                                | Link |
|----------------|---------|--------------------------------|------|
| `clap`        | `4.5.23` | Command-line argument parsing | [📦 Crates.io](https://crates.io/crates/clap) |
| `clipboard-win` | `4.4.1` | Windows clipboard integration | [📦 Crates.io](https://crates.io/crates/clipboard-win) |
| `eframe`      | `0.30.0` | GUI framework for Rust | [📦 Crates.io](https://crates.io/crates/eframe) |
| `rfd`         | `0.15.2` | File dialog UI helper | [📦 Crates.io](https://crates.io/crates/rfd) |
| `windows`     | `0.59.0` | Windows API bindings | [📦 Crates.io](https://crates.io/crates/windows) |
| `winit`       | `0.30.8` | Window handling library | [📦 Crates.io](https://crates.io/crates/winit) |

---

## 🚀 Usage  

### **Launching the GUI Mode**  
Simply run the executable to open the graphical user interface:

```sh
./code-file-wrapper
```

**GUI Features:**  
✔️ **Select a processing mode**  
✔️ **Enable automatic clipboard saving**  
✔️ **Provide additional multi-line notes**  

---

## 📜 How It Works  

### **Updated Flowchart of Execution**  
```mermaid
graph TD;
    A[Start] --> B[Open GUI]
    B --> C[User Selects Directory]
    C --> D[User Chooses File Type]
    D --> E[User Adds Additional Commands]
    E --> F[Process Files]
    F -->|Valid Files Found| G[Wrap Content in Tags]
    F -->|No Valid Files| H[Show Error Message]
    G --> I[Write to tags_output.txt]
    I --> J[Append Additional Commands]
    J -->|Clipboard Enabled| K[Copy to Clipboard]
    J -->|Clipboard Disabled| L[Skip Copy]
    K --> M[Success: Exit]
    L --> M
```

---

## ➡️ Output Example  
**Generated `tags_output.txt` file:**  
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

## 📊 Example Use Cases  

### 📝 **AI Context Provider**
- Useful for AI models like ChatGPT when asking for **code improvements or bug fixes**.  
- Helps **preserve file structure** by wrapping content inside **tagged blocks**.

### 🔍 **Automated Documentation**
- Can assist in **automatically extracting** relevant portions of code for documentation.  
- Provides a **consistent format** that can be parsed by other scripts.

### 💾 **Clipboard Integration**
- Allows quick **copy-pasting** of formatted code content into emails, documentation, or issue reports.

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

If you like this project, **drop a ⭐ on GitHub!** 🚀  
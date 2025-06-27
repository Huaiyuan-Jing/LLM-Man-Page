# llm-man-page

> **Better man pages, powered by LLM**

`llm-man-page` is a CLI tool that provides more readable and modern command documentation compared to the traditional `man` command.  
It automatically fetches the official man page for any command and uses GPT to rewrite it in a clearer, friendlier way — perfect for both beginners and experienced users.

---

## ✨ Features

- Supports most Linux commands (no need for local man pages)
- Uses OpenAI GPT to rewrite documentation for improved readability
- Custom prompts/questions supported
- API Key securely stored on your machine
- Works on Linux and macOS (Rust required)

---

## 🚀 Quick Start

### 1. Install

You need [Rust](https://www.rust-lang.org/) installed.

```sh
git clone https://github.com/yourname/llm-man-page.git
cd llm-man-page
cargo build --release
```

### 2. Configure OpenAI API Key

Set your OpenAI API key the first time (only needed once):

```sh
llman --key sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

```

### 3. Usage

Query an LLM-enhanced man page

```sh
llman ls
llman cat
llman grep
```

Result: Directly outputs a more readable man page for the command.

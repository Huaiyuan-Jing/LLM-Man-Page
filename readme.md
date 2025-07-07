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

**Note**: `install.sh` has only been tested on Fedora Workstation 42, but should work on other Linux distributions.

You need [Rust](https://www.rust-lang.org/) installed.

First clone the repository and change current working directory:

```sh
git clone https://github.com/Huaiyuan-Jing/LLM-Man-Page
cd LLM-Man-Page
```

To just install for the current user (provided that Rust is installed to `~/.cargo/`, which is the default setup with official Rust installation script):

```sh
./install.sh
```

`llman` binary will be installed to `~/.cargo/bin/`

To install for all users (requires root privilege):

```sh
./install.sh --system
```

To install with debug info (can combine with `--system` option):

```sh
./install.sh --debug
```

Tips: run `./install.sh --help` to explore more options

### 2. Configure Engine and Model

Set the LLM service, replace `<name-of-service>` with 'openai' or 'ollama':

```sh
llman --engine <name-of-service>

```

Set the model, replace `<name-of-model>` with the model that your service support, like 'gpt-4-turbo' for 'openai'

```sh
llman --model <name-of-model>
```

If you use 'openai' as LLM service, you need to setup your api key by following command, replace `<key>` with your openai api key like 'sk-xxxxxxxxxxxxxxxxxxxx':

```sh
llman --key <key>
```

### 3. Usage

Query an LLM-enhanced man page

```sh
llman ls
llman cat
llman grep
```

Result: Directly outputs a more readable man page for the command.

### 4. Uninstall

We recommend you to work with `install.sh` to uninstall `llman`, if you have no idea about what happened during installation.

If you installed only for the current user, run:

```sh
./install.sh --uninstall
```

`llman` will be removed from `~/.cargo/bin/`

If you installed for all users, run (requires root privilege):

```sh
./install.sh --uninstall --system
```

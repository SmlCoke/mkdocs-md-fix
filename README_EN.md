# mkdocs-md-fix

[中文](README.md) | English

**Mkdocs Markdown Fixer (mmf)** is a Rust CLI that checks and fixes Markdown block spacing, indentation and abnormal prose spaces that may look correct in an editor preview but render incorrectly in MkDocs.

Current version: **0.1.1**. It includes the command-line executable `mmf` and the interactive executable `mmf_shell`, under the MIT license. MkDocs is not required to run the tool. Rendering still depends on your project's Markdown extensions and configuration.

## Supported rules

[docs/spec.md](docs/spec.md) is the rule specification, with correct and incorrect examples (in Chinese).

| Syntax | Check and fix |
| --- | --- |
| Lists | Add blank lines before and after list blocks |
| Nested lists | Normalize indentation to multiples of four spaces, rounding up |
| Block quotes in list items | Use the recommended four-space indentation and surrounding blank lines |
| Fenced code blocks in list items | Supply the required four-space indentation level and surrounding blank lines |
| Display math | Surround both single-line `$$...$$` and multiline `$$` blocks with blank lines |
| Fenced code blocks | Add blank lines around the whole block |
| Markdown tables | Add blank lines around the table block |
| Prose spaces | Remove spaces between Chinese characters/punctuation and collapse repeated prose spaces to one |

Spaces inside inline code, fenced code and math are preserved. No missing spaces are inserted between Chinese, English or numbers. Structural indentation, list-marker padding and hard line breaks are preserved. Inline math has no additional formatting rules. Markdown examples inside fenced code blocks are left alone, except for required indentation when the block is inside a list. Table detection covers consecutive lines starting and ending with `|`. The tool uses text rules rather than a full Markdown syntax tree; it does not provide general Markdown validation or verify a MkDocs build.

## Installation and stable versions

### Download a GitHub Release (no Rust required)

Open the [v0.1.1 Release](https://github.com/SmlCoke/mkdocs-md-fix/releases/tag/v0.1.1) and download the archive for your operating system and CPU:

| System / architecture | Archive |
| --- | --- |
| Windows x64 | `mmf-v0.1.1-x86_64-pc-windows-msvc.zip` |
| Linux x64 | `mmf-v0.1.1-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `mmf-v0.1.1-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `mmf-v0.1.1-aarch64-apple-darwin.tar.gz` |

Each archive includes `mmf`, `mmf_shell` (with `.exe` on Windows), both READMEs, both usage guides, the specification and the license. The Linux package is built and tested on Ubuntu 22.04 for compatible glibc systems. macOS packages use a minimum deployment target of macOS 11.0. Build from source for other platforms or architectures.

Windows example, from the download directory:

```powershell
Expand-Archive .\mmf-v0.1.1-x86_64-pc-windows-msvc.zip -DestinationPath D:\Tools
& D:\Tools\mmf-v0.1.1-x86_64-pc-windows-msvc\mmf.exe help
```

Add the extracted version directory, such as `D:\Tools\mmf-v0.1.1-x86_64-pc-windows-msvc`, to your user PATH and reopen the terminal. Remove any repository `target\debug` entry or put the stable directory first. Use `Get-Command mmf` to verify which executable is selected.

Linux example (substitute the matching archive and directory on macOS):

```sh
mkdir -p "$HOME/.local/opt"
tar -xzf mmf-v0.1.1-x86_64-unknown-linux-gnu.tar.gz -C "$HOME/.local/opt"
export PATH="$HOME/.local/opt/mmf-v0.1.1-x86_64-unknown-linux-gnu:$PATH"
mmf help
```

Add the PATH setting to your shell configuration to make it persistent. The Release also includes explicit source `.zip` / `.tar.gz` archives, a standalone `spec.md`, and `SHA256SUMS`. To verify a download:

```powershell
Get-FileHash .\mmf-v0.1.1-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

```sh
sha256sum mmf-v0.1.1-x86_64-unknown-linux-gnu.tar.gz
# macOS: shasum -a 256 mmf-v0.1.1-aarch64-apple-darwin.tar.gz
```

Compare the result with the matching filename in `SHA256SUMS`.

### Install a specific tag (Rust required)

Install the [Rust toolchain](https://www.rust-lang.org/tools/install), then run:

```sh
cargo install --git https://github.com/SmlCoke/mkdocs-md-fix --tag v0.1.1 --locked --bins
```

Both executables are installed in the installation root's `bin` directory, usually `$HOME/.cargo/bin`, or under `CARGO_HOME` when configured. To use a separate installation directory:

```sh
cargo install --git https://github.com/SmlCoke/mkdocs-md-fix --tag v0.1.1 --locked --bins --root ./mmf-0.1.1
```

Add its `bin` subdirectory to PATH. A tagged installation is independent of subsequent builds in your development checkout; it changes only when you explicitly reinstall or replace it. See the [official Cargo installation reference](https://doc.rust-lang.org/cargo/commands/cargo-install.html) for the options.

### Build from source

```sh
git clone https://github.com/SmlCoke/mkdocs-md-fix.git
cd mkdocs-md-fix
git checkout v0.1.1
cargo test --locked
cargo build --release --locked --bins
```

Executables are written to `target/release/`. Run `./target/release/mmf help` on Linux/macOS or `.\target\release\mmf.exe help` in Windows PowerShell. This build directory also changes on rebuilds. Copy both executables to a separate version directory for everyday use, or install from the tag as shown above.

The maintainer uses MSYS2 CLANG64 locally, with Cargo under `/clang64/.cargo` and rustup under `/clang64/.rustup`. Run the same Cargo commands inside that environment. Release users do not need MSYS2.

## Quick start

### Command-line mode

```sh
mmf check notes.md
mmf check docs notes.md -r
mmf fix notes.md
mmf write docs -r
mmf help
```

`check` reports issues with original line numbers. `fix` prints a colored diff for one file. `write` prints the diff and overwrites files. Directories process only `.md` files in the current directory unless `-r` is given. Check first, preview with `fix`, then apply with `write`. No automatic backup is created; use Git to review and recover changes.

For example, given `notes.md`:

```markdown
Introduction:
- First item
  - Child item
End.
```

`mmf write notes.md` produces:

```markdown
Introduction:

- First item
    - Child item

End.
```

### Interactive shell

Run `mmf_shell`, then enter commands without the `mmf` prefix:

```text
mmf>check docs/spec.md
mmf>fix "notes/my file.md"
mmf>write docs -r
mmf>help
mmf>exit
```

For all commands, path handling, exit codes, Windows examples and details, see the [English usage guide](docs/usage_EN.md) / [中文命令手册](docs/usage.md).

## Repository layout

```text
mkdocs-md-fix/
├── .github/
│   ├── workflows/release.yml      # Native tests, builds and artifact uploads
│   └── scripts/package_release.py # Package and verify binaries and documentation
├── src/
│   ├── lib.rs                     # Module declarations and public exports
│   ├── markdown.rs                # Markdown fixing pipeline
│   ├── model.rs                   # Issue / FixResult types
│   ├── syntax.rs                  # Shared line, list and fence recognition
│   ├── syntax/protected.rs        # Code, math and structural whitespace protection
│   ├── rules/                     # Rules grouped by responsibility
│   │   ├── mod.rs                 # Rule dispatch and block boundary helpers
│   │   ├── lists.rs               # List spacing and nested indentation
│   │   ├── quotes.rs              # Quotes inside lists
│   │   ├── code.rs                # Fenced code blocks
│   │   ├── math.rs                # Display math spacing
│   │   ├── tables.rs              # Table spacing
│   │   └── spacing.rs             # Abnormal prose spaces (spec 5.1)
│   ├── cli.rs                     # Command parsing and execution
│   ├── files.rs                   # Input collection and traversal
│   ├── diff.rs                    # Colored diff computation and output
│   ├── shell.rs                   # Interactive input tokenization
│   ├── regression_tests.rs        # Existing behavior regression tests
│   ├── main.rs                    # mmf entry point
│   └── bin/mmf_shell.rs           # Interactive loop and welcome banner
├── docs/
│   ├── spec.md                    # Markdown rule specification
│   ├── usage.md                   # Chinese command reference
│   └── usage_EN.md                # English command reference
├── AGENTS.md                     # Agent working rules in Chinese
├── Cargo.toml                    # Package version, library and binary targets
├── Cargo.lock                    # Locked package version information
├── README.md                     # Chinese project documentation
├── README_EN.md                  # English project documentation
├── LICENSE                       # MIT license
├── .gitignore                    # Ignore build output and local development files
├── target/                       # Local build output, not tracked
└── dist/                         # Local release artifacts, not tracked
```

`docs/prompts/` contains ignored local task materials and is excluded from source releases. Existing regression tests live in `src/regression_tests.rs`; new rule tests live alongside their rule modules. There is no tracked standalone `tests/` directory. See [AGENTS.md](AGENTS.md) for agent working rules (in Chinese).

## Development and license

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked --bins
```

The manually dispatched [Release build workflow](.github/workflows/release.yml) tests, builds and uploads archives for Windows, Linux and both macOS architectures. It generates artifacts; the maintainer publishes a Release after checking them.

Rule changes should follow the [specification](docs/spec.md). The project is licensed under the [MIT License](LICENSE).

# mkdocs-md-fix

`mmf` is a small Rust CLI for checking and fixing Markdown syntax patterns that commonly render in editor previews but fail or mis-render in MkDocs.

The implemented rules follow [docs/specification.md](docs/specification.md):

- blank line before and after list blocks
- nested list indentation normalized to multiples of four spaces
- blank lines and four-space indentation for block quotes inside list items
- blank lines and four-space indentation for fenced code blocks inside list items
- blank lines around display math blocks
- blank lines around fenced code blocks
- blank lines around Markdown tables

## Build

```powershell
D:\MSYS\msys64\clang64\.cargo\bin\cargo.exe build
```

## CLI Usage

```powershell
mmf check <path> [-r]
mmf fix <file>
mmf write <path> [-r]
mmf help
```

`<path>` may be a Markdown file or a directory. Directories are processed non-recursively by default; add `-r` to include subdirectories.

`fix` prints a colored diff for a single file without modifying it. Use `write` to print the same diff and update files in place.

## Interactive Shell

```powershell
mmf_shell
```

Inside the shell, use the same commands without the `mmf` prefix:

```text
mmf> check docs\specification.md
mmf> fix notes\bad.md
mmf> write docs -r
mmf> exit
```

use std::io::{self, Write};

fn main() {
    print_welcome();

    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("mmf>");
        if io::stdout().flush().is_err() {
            break;
        }

        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => {
                eprintln!("error: failed to read input: {err}");
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "exit" {
            break;
        }

        let args = match mmf::split_command_line(trimmed) {
            Ok(args) => args,
            Err(err) => {
                eprintln!("error: {err}");
                continue;
            }
        };

        if args.is_empty() {
            continue;
        }

        let mut stdout = io::stdout();
        let mut stderr = io::stderr();
        let _ = mmf::run_command(&args, &mut stdout, &mut stderr);
    }
}

fn print_welcome() {
    const WIDTH: usize = 80;
    let lines = [
        "",
        "Mkdocs Markdown Fixer (TM)",
        "mmf",
        "mmf_shell",
        "",
        "A Markdown Linting and Fixing Suite for MkDocs",
        "",
        concat!("Version ", env!("CARGO_PKG_VERSION")),
        "Author: SmlCoke",
        "Repository: https://github.com/SmlCoke/mkdocs-md-fix",
        "Copyright:",
        "",
        "Type help or ? to list commands.",
    ];

    for line in lines {
        println!("{}", center(line, WIDTH));
    }
}

fn center(text: &str, width: usize) -> String {
    if text.len() >= width {
        return text.to_string();
    }

    let left = (width - text.len()) / 2;
    format!("{}{}", " ".repeat(left), text)
}

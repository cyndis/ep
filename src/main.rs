#![allow(unused_must_use)]

extern crate epwing;
extern crate term;

struct Arguments {
    book_path: std::path::PathBuf,
    expand_unknown: bool,
    command: Command,
    format: Format
}

#[derive(Debug)]
enum Command {
    Search(epwing::subbook::Index, String),
}

#[derive(Debug, Copy, Clone)]
enum Format {
    Terminal,
    HTML
}

fn parse_command() -> Option<Arguments> {
    use std::os::unix::ffi::OsStrExt;

    let mut book_path = None;
    let mut query = None;
    let mut expand_unknown = false;

    let args = std::env::args_os();
    let mut iter = args.skip(1);
    let mut format = Format::Terminal;

    while let Some(arg) = iter.next() {
        let arg = arg.as_bytes();
        if arg == b"-b" {
            book_path = Some(iter.next().unwrap());
        } else if arg == b"--expand-unknown-characters" {
            expand_unknown = true;
        } else if arg == b"--html" {
            format = Format::HTML;
        } else {
            query = Some(std::str::from_utf8(arg).unwrap().to_owned());
        }
    }

    let book_path = match book_path {
        Some(p) => p.into(),
        None    => std::env::var("EP_BOOK_PATH").ok().expect("Put book path in $EP_BOOK_PATH")
                                                .into()
    };

    match query {
        Some(query) => {
            Some(Arguments {
                format: format,
                book_path: book_path,
                expand_unknown: expand_unknown,
                command: Command::Search(epwing::subbook::Index::WordAsIs, query)
            })
        },
        _ => None
    }
}

fn convert_custom_character(cp: u16) -> Option<&'static str> {
    match cp {
        0xb667 => Some("[ローマ字]"),
        0xb65e => Some("▶ "),
        0xb66b => Some("◧"),

        0xa239 => Some("ū"),

        _ => None
    }
}

fn print_text(text: &epwing::subbook::Text, expand_unknown: bool, format: Format) {
    use epwing::subbook::TextElement::{UnicodeString, CustomCharacter, Newline, Indent,
                                       NoNewline, BeginDecoration, EndDecoration, Unsupported};

    let mut term = term::stdout().unwrap();

    let nl = match format {
        Format::Terminal => "\n",
        Format::HTML => "<br>"
    };

    for elem in text.iter() {
        match *elem {
            UnicodeString(ref string) => write!(term, "{}", string).unwrap(),
            CustomCharacter(cp)       => {
                match convert_custom_character(cp) {
                    Some(s) => write!(term, "{}", s).unwrap(),
                    None if expand_unknown => write!(term, "<?0x{:4x}>", cp).unwrap(),
                    _ => ()
                }
            },
            Newline                   => write!(term, "{}", nl).unwrap(),
            Indent(n)                 => {
                for _ in 0..n {
                    (write!(term, " ")).unwrap();
                }
            },
            NoNewline(_enabled)       => (),
            BeginDecoration(_deco)    => {
                match format {
                    Format::Terminal => term.attr(term::Attr::Standout(true)).unwrap(),
                    Format::HTML => write!(term, "<b>").unwrap()
                }
            },
            EndDecoration             => {
                match format {
                    Format::Terminal => term.reset().unwrap(),
                    Format::HTML => write!(term, "</b>").unwrap()
                }
            },
            Unsupported(_tag)         => ()
        }
    }
}

fn main() {
    let args = match parse_command() {
        Some(x) => x,
        None    => {
            println!(r"
Usage: ep [options] <search query>
Options:
  -b <book path>                Specify path to EPWING book. Default is environment variable
                                $EP_BOOK_PATH.
  --expand-unknown-characters   Show character codes for characters without Unicode codepoint.
");
            return
        }
    };

    let book = match epwing::Book::open(args.book_path) {
        Ok(x) => x,
        Err(e) => { println!("Failed to open book: {}", e); return }
    };

    let spine = &book.subbooks()[0];
    let mut subbook = book.open_subbook(spine).unwrap();

    let mut term = term::stdout().unwrap();

    match args.command {
        Command::Search(index, query) => {
            let result = subbook.search(index, &query).unwrap();
            for (i, location) in result.iter().enumerate() {
                match args.format {
                    Format::Terminal => {
                        term.attr(term::Attr::Bold);
                        writeln!(term, "-- {} of {} --", i+1, result.len());
                        term.reset();
                    }
                    Format::HTML => {
                        if i > 0 {
                            writeln!(term, "<hr>");
                        }
                        writeln!(term, "<p><b>Entry {} of {}</b></p>", i+1, result.len());
                    }
                }
                let text = subbook.read_text(*location).unwrap();

                print_text(&text, args.expand_unknown, args.format);
            }

            if result.len() == 0 {
                writeln!(term, "No results.");
            }
        }
    }

    writeln!(term, "");

    term.reset();
}

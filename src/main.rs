#![feature(old_io, old_path, os, core)]
#![allow(unused_must_use)]

extern crate epwing;
extern crate term;

struct Arguments {
    book_path: Path,
    expand_unknown: bool,
    command: Command
}

#[derive(Debug)]
enum Command {
    Search(epwing::subbook::Index, String),
}

fn parse_command() -> Option<Arguments> {
    let mut book_path = None;
    let mut query = None;
    let mut expand_unknown = false;

    let args = std::os::args();
    let mut iter = args.into_iter().skip(1);

    while let Some(arg) = iter.next() {
        if arg == "-b" {
            book_path = Some(iter.next().unwrap());
        } else if arg == "--expand-unknown-characters" {
            expand_unknown = true;
        } else {
            query = Some(arg);
        }
    }

    let book_path = match book_path {
        Some(p) => Path::new(p),
        None    => Path::new(std::env::var("EP_BOOK_PATH").ok().
                                                           expect("Put book path in $EP_BOOK_PATH"))
    };

    match query {
        Some(query) => {
            Some(Arguments {
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

fn print_text(text: &epwing::subbook::Text, expand_unknown: bool) {
    use epwing::subbook::TextElement::{UnicodeString, CustomCharacter, Newline, Indent,
                                       NoNewline, BeginDecoration, EndDecoration, Unsupported};

    let mut term = term::stdout().unwrap();

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
            Newline                   => write!(term, "\n").unwrap(),
            Indent(n)                 => {
                for _ in range(0, n) {
                    (write!(term, " ")).unwrap();
                }
            },
            NoNewline(_enabled)       => (),
            BeginDecoration(_deco)    => { term.attr(term::Attr::Standout(true)); },
            EndDecoration             => { term.reset(); },
            Unsupported(_tag)         => ()
        }
    }
}

fn main() {
    let args = match parse_command() {
        Some(x) => x,
        None    => {
            println!(r"
Usage: ep [options][-b <book path>] <search query>
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
                term.attr(term::Attr::Bold);
                writeln!(term, "-- {} of {} --", i+1, result.len());
                term.reset();
                let text = subbook.read_text(*location).unwrap();

                print_text(&text, args.expand_unknown);
            }

            if result.len() == 0 {
                writeln!(term, "No results.");
            }
        }
    }

    writeln!(term, "");

    term.reset();
}

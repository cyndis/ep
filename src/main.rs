#![feature(while_let, slicing_syntax)]
#![allow(unused_must_use)]

extern crate epwing;
extern crate term;

struct Arguments {
    book_path: Path,
    command: Command
}

#[deriving(Show)]
enum Command {
    Search(epwing::subbook::Index, String),
}

static DEFAULT_BOOK_PATH: &'static str = "/home/cyndis/data/kenkyusha/enja";

fn parse_command() -> Option<Arguments> {
    let mut book_path = None;
    let mut query = None;

    let args = std::os::args();
    let mut iter = args.into_iter().skip(1);

    while let Some(arg) = iter.next() {
        if arg[] == "-b" {
            book_path = Some(iter.next().unwrap());
        } else {
            query = Some(arg);
        }
    }

    let book_path = match book_path {
        Some(p) => Path::new(p),
        None    => Path::new(DEFAULT_BOOK_PATH)
    };

    match query {
        Some(query) => {
            Some(Arguments {
                book_path: book_path,
                command: Command::Search(epwing::subbook::Index::WordAsIs, query)
            })
        },
        _ => None
    }
}

fn print_text(text: &epwing::subbook::Text) {
    use epwing::subbook::TextElement::{UnicodeString, CustomCharacter, Newline, Indent,
                                       NoNewline, BeginDecoration, EndDecoration, Unsupported};

    let mut term = term::stdout().unwrap();

    for elem in text.iter() {
        match *elem {
            UnicodeString(ref string) => write!(term, "{}", string).unwrap(),
            CustomCharacter(_)        => (),
            Newline                   => write!(term, "\n").unwrap(),
            Indent(n)                 => {
                for _ in range(0, n) {
                    (write!(term, " ")).unwrap();
                }
            },
            NoNewline(_enabled)       => (),
            BeginDecoration(_deco)    => { term.attr(term::attr::Attr::Bold); },
            EndDecoration             => { term.reset(); },
            Unsupported(_tag)         => ()
        }
    }
}

fn main() {
    let args = match parse_command() {
        Some(x) => x,
        None    => panic!("Invalid usage")
    };

    let book = match epwing::Book::open(args.book_path) {
        Ok(x) => x,
        Err(e) => { println!("Failed to open book: {}", e); return }
    };

    let spine = book.subbooks().head().unwrap();
    let mut subbook = book.open_subbook(spine).unwrap();

    let mut term = term::stdout().unwrap();

    match args.command {
        Command::Search(index, query) => {
            let result = subbook.search(index, query[]).unwrap();
            for (i, location) in result.iter().enumerate() {
                term.attr(term::attr::Attr::Bold);
                writeln!(term, "-- {} of {} --", i+1, result.len());
                term.reset();
                let text = subbook.read_text(*location).unwrap();

                print_text(&text);
            }

            if result.len() == 0 {
                writeln!(term, "No results.");
            }
        }
    }

    writeln!(term, "");

    term.reset();
}

use std::collections::HashSet;

use logos::skip;
use logos::Lexer;
use logos::Logos;

struct Extras<'a> {
    emotes: HashSet<&'a str>,
}

#[derive(Logos, Debug)]
#[logos(extras = Extras<'s>)]
enum Part {
    #[regex(r"\w+", emote)]
    Emote,

    #[regex(r"https?", url)]
    Url,

    #[regex(r"\s+", skip)]
    Whitespace,

    #[error]
    Text,
}

fn emote(l: &mut Lexer<'_, Part>) -> Option<()> {
    if l.extras.emotes.contains(l.slice()) {
        Some(())
    } else {
        None
    }
}

fn url(l: &mut Lexer<'_, Part>) -> Option<()> {
    let i = l
        .remainder()
        .find(|c: char| c.is_ascii_whitespace())
        .unwrap_or(l.remainder().len());
    l.bump(i);
    url::Url::parse(l.slice()).ok().map(|_| ())
}

fn main() {
    let input = "test asdf abcd ok http://test.com/a?b=c";
    let extras = Extras {
        emotes: HashSet::from_iter(["test", "ok"]),
    };

    let mut lexer = Part::lexer_with_extras(input, extras);
    let mut parts = vec![];
    while let Some(token) = lexer.next() {
        parts.push((token, lexer.span(), lexer.slice()))
    }

    println!("{parts:?}");
}

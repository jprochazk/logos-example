use std::borrow::Cow;
use std::collections::HashSet;

use logos::Lexer;
use logos::Logos;
use logos::Span;
use std::iter::Peekable;

#[derive(Default, Clone)]
pub struct Extras {
    pub emotes: HashSet<String>,
    pub names: HashSet<String>,
}

#[derive(Logos, PartialEq, Eq, Debug)]
#[logos(extras = Extras)]
pub enum PartKind {
    #[regex(r"https?", url)]
    Url,
    #[regex(r"([^\s][^\s][^\s]?)|[\w\d]+", emote)]
    Emote,
    #[regex(r"```[^`]+```")]
    #[regex(r"`[^`]+`")]
    Code,
    #[regex(r"@\w+", mention)]
    Mention,
    #[error]
    Text,
}

#[derive(Debug)]
pub struct Part<'src> {
    pub str: Cow<'src, str>,
    pub span: Span,
    pub kind: PartKind,
}

fn emote(l: &mut Lexer<'_, PartKind>) -> Option<()> {
    eprintln!("emote");
    if l.extras.emotes.contains(l.slice()) {
        Some(())
    } else {
        None
    }
}

fn url(l: &mut Lexer<'_, PartKind>) -> Option<()> {
    eprintln!("url");
    let i = l
        .remainder()
        .find(|c: char| c.is_ascii_whitespace())
        .unwrap_or(l.remainder().len());
    l.bump(i);
    url::Url::parse(l.slice()).ok().map(|_| ())
}

fn mention(l: &mut Lexer<'_, PartKind>) -> Option<()> {
    if l.extras
        .names
        .contains(l.slice().strip_prefix('@').unwrap())
    {
        Some(())
    } else {
        None
    }
}

struct Parts<'src> {
    inner: Lexer<'src, PartKind>,
}

impl<'src> Iterator for Parts<'src> {
    type Item = Part<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|kind| Part {
            str: self.inner.slice().into(),
            span: self.inner.span(),
            kind,
        })
    }
}

pub struct Parser<'src> {
    inner: Peekable<Parts<'src>>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str, extras: Extras) -> Self {
        Self {
            inner: Parts {
                inner: Lexer::with_extras(src, extras),
            }
            .peekable(),
        }
    }
}

fn try_combine_text<'src>(p: &mut Parser<'src>, next: &mut Part<'src>) {
    // at least 2 Text matches in a row
    if matches!(
        next,
        Part {
            kind: PartKind::Text,
            ..
        }
    ) && matches!(
        p.inner.peek(),
        Some(Part {
            kind: PartKind::Text,
            ..
        })
    ) {
        // combine spans and strings
        let mut span = next.span.clone();
        let mut str = next.str.to_string();
        while matches!(
            p.inner.peek(),
            Some(Part {
                kind: PartKind::Text,
                ..
            })
        ) {
            let temp = p.inner.next().unwrap();
            span.end = temp.span.end;
            str += &temp.str;
        }
        next.span = span;
        next.str = str.into();
    }
}

impl<'src> Iterator for Parser<'src> {
    type Item = Part<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut next) = self.inner.next() {
            try_combine_text(self, &mut next);

            Some(next)
        } else {
            None
        }
    }
}

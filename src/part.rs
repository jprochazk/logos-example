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
    // Emotes and URLs are processed later
    Url,
    Emote,
    #[regex(r"```[^`]+```")]
    #[regex(r"`[^`]+`")]
    Code,
    #[regex(r"@\w+", mention)]
    Mention,
    #[regex(r"[!-~]+")]
    #[error]
    Text,
}

#[derive(Debug)]
pub struct Part<'src> {
    pub str: Cow<'src, str>,
    pub span: Span,
    pub kind: PartKind,
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

struct Emotes<'src> {
    inner: Parts<'src>,
}

impl<'src> Iterator for Emotes<'src> {
    type Item = Part<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|part| {
            if self.inner.inner.extras.emotes.contains(part.str.as_ref()) {
                Part {
                    kind: PartKind::Emote,
                    ..part
                }
            } else {
                part
            }
        })
    }
}

struct Urls<'src> {
    inner: Emotes<'src>,
}

impl<'src> Iterator for Urls<'src> {
    type Item = Part<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|part| {
            if part.kind == PartKind::Text && url::Url::parse(&part.str).is_ok() {
                Part {
                    kind: PartKind::Url,
                    ..part
                }
            } else {
                part
            }
        })
    }
}

pub struct Parser<'src> {
    src: &'src str,
    inner: Peekable<Urls<'src>>,
}

impl<'src> Parser<'src> {
    pub fn new(src: &'src str, extras: Extras) -> Self {
        let inner = Lexer::with_extras(src, extras);
        let inner = Parts { inner };
        let inner = Emotes { inner };
        let inner = Urls { inner };
        let inner = inner.peekable();
        Self { src, inner }
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
        // extend the current parts's span to also contain all the subsequent text parts
        let mut span = next.span.clone();
        while matches!(
            p.inner.peek(),
            Some(Part {
                kind: PartKind::Text,
                ..
            })
        ) {
            let temp = p.inner.next().unwrap();
            span.end = temp.span.end;
        }
        // use that extended span to slice the original source
        next.span = span.clone();
        next.str = p.src[span].into();
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

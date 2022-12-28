use std::fmt;

use super::*;

fn parse(input: &str, extras: Extras) -> Vec<IgnoreSpan> {
    Parser::new(input, extras).map(IgnoreSpan).collect()
}

struct IgnoreSpan<'src>(Part<'src>);
impl<'src> PartialEq<IgnoreSpan<'src>> for IgnoreSpan<'src> {
    fn eq(&self, other: &IgnoreSpan<'src>) -> bool {
        self.0.kind == other.0.kind && self.0.str == other.0.str
    }
}
impl<'src> fmt::Debug for IgnoreSpan<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Part")
            .field(&self.0.kind)
            .field(&self.0.str)
            .finish()
    }
}

macro_rules! part {
    ($kind:ident, $text:expr) => {
        IgnoreSpan(Part {
            kind: PartKind::$kind,
            str: $text.into(),
            span: 0..0,
        })
    };
}

#[test]
fn simple_code() {
    let message = r#"```let foo = $"bar";```"#;

    let parts = parse(message, Extras::default());

    assert_eq!(vec![part!(Code, r#"```let foo = $"bar";```"#)], parts);
}

#[test]
fn simple_text() {
    let message = r#"Hello, this is a test!"#;

    let parts = parse(message, Extras::default());

    assert_eq!(vec![part!(Text, "Hello, this is a test!"),], parts);
}

#[test]
fn text_and_code() {
    let message = r#"Hello, ```let x = 3;``` this is a test!"#;

    let parts = parse(message, Extras::default());

    assert_eq!(
        parts,
        vec![
            part!(Text, "Hello, "),
            part!(Code, "```let x = 3;```"),
            part!(Text, " this is a test!"),
        ]
    );
}

#[test]
fn twitch_message_to_parts_basic() {
    let actual = parse(
        "Hello 👪 Kappa World!",
        Extras {
            emotes: HashSet::from(["Kappa".into()]),
            ..Default::default()
        },
    );

    let expected = vec![
        part!(Text, "Hello 👪 "),
        part!(Emote, "Kappa"),
        part!(Text, " World!"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn twitch_message_to_parts_single_backtick() {
    let actual = parse("test `", Extras::default());

    let expected = vec![part!(Text, "test `")];

    assert_eq!(actual, expected);
}

#[test]
fn twitch_message_to_parts_code() {
    let actual = parse("Hello `👪 Kappa ` World!", Extras::default());

    let expected = vec![
        part!(Text, "Hello "),
        part!(Code, r#"`👪 Kappa `"#),
        part!(Text, " World!"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn twitch_message_to_parts_code_escaping() {
    let actual = parse("Hello `👪 Kappa <script> ` World!", Extras::default());

    let expected = vec![
        part!(Text, "Hello "),
        part!(Code, "`👪 Kappa <script> `"),
        part!(Text, " World!"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn twitch_message_to_parts_fenced_code_escaping() {
    let actual = parse("Hello ```👪 Kappa <script> ``` World!", Extras::default());

    let expected = vec![
        part!(Text, "Hello "),
        part!(Code, "```👪 Kappa <script> ```"),
        part!(Text, " World!"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn twitch_message_to_parts_code_and_emote() {
    let actual = parse(
        "Hello `👪 Kappa` Kappa World!",
        Extras {
            emotes: HashSet::from(["Kappa".into()]),
            ..Default::default()
        },
    );

    let expected = vec![
        part!(Text, "Hello "),
        part!(Code, r#"`👪 Kappa`"#),
        part!(Text, " "),
        part!(Emote, "Kappa"),
        part!(Text, " World!"),
    ];

    assert_eq!(actual, expected);
}
#[test]
fn twitch_message_to_parts_emotes() {
    let actual = parse(
        "mrhalzClippy LUL mrhalzClippy",
        Extras {
            emotes: HashSet::from(["mrhalzClippy".into(), "LUL".into()]),
            ..Default::default()
        },
    );

    let expected = vec![
        part!(Emote, "mrhalzClippy"),
        part!(Text, " "),
        part!(Emote, "LUL"),
        part!(Text, " "),
        part!(Emote, "mrhalzClippy"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn twitch_message_to_parts_subscriber_emotes() {
    let actual = parse(
        "breade3Box icedkiSleep idlefaBlankies",
        Extras {
            emotes: HashSet::from([
                "breade3Box".into(),
                "icedkiSleep".into(),
                "idlefaBlankies".into(),
            ]),
            ..Default::default()
        },
    );

    let expected = vec![
        part!(Emote, "breade3Box"),
        part!(Text, " "),
        part!(Emote, "icedkiSleep"),
        part!(Text, " "),
        part!(Emote, "idlefaBlankies"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn ascii_emotes() {
    let emotes = vec![
        "8-)", ":(", ":)", ":-(", ":-)", ":-/", ":-D", ":-O", ":-P", ":-Z", ":-", ":-o", ":-p",
        ":-z", ":-|", ":/", ":D", ":O", ":P", ":Z", ":-\\", ":o", ":p", ":z", ":|", ":|", ";)",
        ";)", ";-)", ";-P", ";-p", ";P", ";p", ">(", "<3", "B)", "B-)", "O_O", "O_o", "R)", "R-)",
        "o_O", "o_o",
    ];

    let extras = Extras {
        emotes: HashSet::from_iter(emotes.iter().map(|&s| s.to_owned())),
        ..Default::default()
    };

    for emote in emotes {
        let actual = parse(emote, extras.clone()).remove(0);

        let expected = part!(Emote, emote);

        assert_eq!(actual, expected, "failed {emote}");
    }
}

#[test]
fn url() {
    let actual = parse(
        "https://github.com/jprochazk/logos-example/blob/master/src/main.rs not sure",
        Extras::default(),
    );

    let expected = vec![
        part!(
            Url,
            "https://github.com/jprochazk/logos-example/blob/master/src/main.rs"
        ),
        part!(Text, " not sure"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn mentions() {
    let actual = parse(
        "hey @mrhalzy",
        Extras {
            names: ["mrhalzy".into()].into_iter().collect(),
            ..Default::default()
        },
    );

    let expected = vec![part!(Text, "hey "), part!(Mention, "@mrhalzy")];

    assert_eq!(actual, expected);
}

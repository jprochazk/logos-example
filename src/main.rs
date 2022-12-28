mod part;

use std::collections::HashSet;

use part::*;

fn main() {
    let input = "https://github.com/jprochazk/logos-example/blob/master/src/main.rs not sure";
    let extras = Extras {
        emotes: HashSet::from_iter(
            [
                "test", "ok", ":)", ":(", ":D", ">(", ":|", "O_o", "B)", ":O", "<3", ":/", ";)",
                ":P", ";P", "R)",
            ]
            .map(String::from),
        ),
        names: HashSet::from_iter(["moscowwbish", "mrhalzy"].map(String::from)),
    };

    let parts = Parser::new(input, extras).collect::<Vec<_>>();

    println!("{parts:?}");
}

#[cfg(test)]
mod tests;

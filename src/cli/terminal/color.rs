use std::fmt;

use owo_colors::{OwoColorize, Stream::Stdout};

macro_rules! trait_methods {
    ($( $color:ident ),* $(,)?) => {
        $(
            fn $color(&self) -> String;
         )*
    };
}

#[allow(dead_code)]
pub trait Colorize {
    trait_methods!(
        black,
        red,
        green,
        yellow,
        blue,
        magenta,
        cyan,
        white,
        bright_black,
        bright_red,
        bright_green,
        bright_yellow,
        bright_blue,
        bright_magenta,
        bright_cyan,
        bright_white,
    );
}

macro_rules! display_methods {
    ($( $color:ident ),* $(,)?) => {
        $(
            fn $color(&self) -> String {
                self.if_supports_color(Stdout, OwoColorize::$color).to_string()
            }
         )*
    };
}

impl<D: fmt::Display> Colorize for D {
    display_methods!(
        black,
        red,
        green,
        yellow,
        blue,
        magenta,
        cyan,
        white,
        bright_black,
        bright_red,
        bright_green,
        bright_yellow,
        bright_blue,
        bright_magenta,
        bright_cyan,
        bright_white,
    );
}

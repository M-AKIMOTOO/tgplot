use std::io::{self, IsTerminal};

use mdlite::render_markdown;

const DETAIL: &str = include_str!("../README.md");

pub(crate) fn print_detail() {
    let color = io::stdout().is_terminal();
    print!("{}", render_markdown(DETAIL, color));
}

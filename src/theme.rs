use ansi_term::Colour;
use ansi_term::Style;

pub enum Type {
    Error,
    Success,
    Warning,
    Info,
    Header,
}

pub fn colorize(style: Type, string: &str) -> String {
    let colored_string = match style {
        Type::Error => Colour::Red.bold().paint(string),
        Type::Success => Colour::Green.bold().paint(string),
        Type::Warning => Colour::Yellow.paint(string),
        Type::Info => Colour::Blue.bold().paint(string),
        Type::Header => Style::new().bold().paint(string),
    };

    colored_string.to_string()
}

use escpos::utils::Font;
use escpos::utils::JustifyMode;
use escpos::utils::UnderlineMode;
use nom::branch::alt;
use nom::bytes::complete::escaped_transform;
use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::bytes::complete::take_while_m_n;
use nom::bytes::is_not;
use nom::character::complete::line_ending;
use nom::character::complete::space0;
use nom::character::complete::space1;
use nom::character::complete::u8;
use nom::combinator::eof;
use nom::combinator::map;
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::sequence::terminated;
use nom::AsChar;
use nom::IResult;
use nom::Parser;
use std::vec::Vec;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub commands: Vec<Command>,
}

impl Program {
    pub fn parse(input: &str) -> IResult<&str, Program> {
        let (remains, commands) = many0(preceded(
            space0,
            terminated(Command::parse, alt((eof, line_ending))),
        ))
        .parse(input)?;

        Ok((remains, Program { commands }))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Write(String),
    Bold(bool),
    Underline(UnderlineMode),
    DoubleStrike(bool),
    Font(Font),
    Flip(bool),
    Justify(JustifyMode),
    Reverse(bool),
    Feed(u8),
    Ean13(String),
    Ean8(String),
    QrCode(String),
    Size(u8, u8),
    ResetSize,
    Sudoku,
    MiniCrossword,
    Cut,
}

fn escaped_string(input: &str) -> IResult<&str, String> {
    delimited(
        tag("\""),
        escaped_transform(
            is_not("\\\""),
            '\\',
            alt((value("\\", tag("\\")), value("\"", tag("\"")))),
        ),
        tag("\""),
    )
    .parse(input)
}

fn bool(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag_no_case("true"), |_| true),
        map(tag_no_case("false"), |_| false),
    ))
    .parse(input)
}

fn underline_mode(input: &str) -> IResult<&str, UnderlineMode> {
    alt((
        map(tag_no_case("none"), |_| UnderlineMode::None),
        map(tag_no_case("single"), |_| UnderlineMode::Single),
        map(tag_no_case("double"), |_| UnderlineMode::Double),
    ))
    .parse(input)
}

fn font(input: &str) -> IResult<&str, Font> {
    alt((
        map(tag_no_case("a"), |_| Font::A),
        map(tag_no_case("b"), |_| Font::B),
        map(tag_no_case("c"), |_| Font::C),
    ))
    .parse(input)
}

fn justify_mode(input: &str) -> IResult<&str, JustifyMode> {
    alt((
        map(tag_no_case("left"), |_| JustifyMode::LEFT),
        map(tag_no_case("center"), |_| JustifyMode::CENTER),
        map(tag_no_case("right"), |_| JustifyMode::RIGHT),
    ))
    .parse(input)
}

impl Command {
    fn parse(input: &str) -> IResult<&str, Command> {
        alt((
            map(
                preceded(pair(tag("write"), space1), escaped_string),
                Command::Write,
            ),
            map(
                preceded(pair(tag("writeln"), space1), escaped_string),
                |string| Command::Write(string + "\n"),
            ),
            map(preceded(pair(tag("bold"), space1), bool), Command::Bold),
            map(
                preceded(pair(tag("underline"), space1), underline_mode),
                Command::Underline,
            ),
            map(
                preceded(pair(tag("double_strike"), space1), bool),
                Command::DoubleStrike,
            ),
            map(preceded(pair(tag("font"), space1), font), Command::Font),
            map(preceded(pair(tag("flip"), space1), bool), Command::Flip),
            map(
                preceded(pair(tag("justify"), space1), justify_mode),
                Command::Justify,
            ),
            map(
                preceded(pair(tag("reverse"), space1), bool),
                Command::Reverse,
            ),
            map(preceded(pair(tag("feed"), space1), u8), Command::Feed),
            map(tag("feed"), |_| Command::Feed(1)),
            map(
                preceded(
                    pair(tag("ean13"), space1),
                    take_while_m_n(12, 13, AsChar::is_dec_digit),
                ),
                |digits| Command::Ean13(String::from(digits)),
            ),
            map(
                preceded(
                    pair(tag("ean8"), space1),
                    take_while_m_n(7, 8, AsChar::is_dec_digit),
                ),
                |digits| Command::Ean8(String::from(digits)),
            ),
            map(
                preceded(pair(tag("qr_code"), space1), escaped_string),
                Command::QrCode,
            ),
            map(
                preceded(pair(tag("size"), space1), separated_pair(u8, tag(","), u8)),
                |(a, b)| Command::Size(a, b),
            ),
            map(tag("reset_size"), |_| Command::ResetSize),
            map(tag("sudoku"), |_| Command::Sudoku),
            map(tag("minicrossword"), |_| Command::MiniCrossword),
            map(tag("cut"), |_| Command::Cut),
        ))
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(
            Command::parse("writeln \"rofl\""),
            Ok(("", Command::Write(String::from("rofl\n"))))
        );
        assert_eq!(Command::parse("bold true"), Ok(("", Command::Bold(true))));
        assert_eq!(Command::parse("bold false"), Ok(("", Command::Bold(false))));
        assert_eq!(
            Command::parse("underline none"),
            Ok(("", Command::Underline(UnderlineMode::None)))
        );
        assert_eq!(
            Command::parse("underline single"),
            Ok(("", Command::Underline(UnderlineMode::Single)))
        );
        assert_eq!(
            Command::parse("underline double"),
            Ok(("", Command::Underline(UnderlineMode::Double)))
        );
        assert_eq!(Command::parse("font a"), Ok(("", Command::Font(Font::A))));
        assert_eq!(Command::parse("font b"), Ok(("", Command::Font(Font::B))));
        assert_eq!(Command::parse("font c"), Ok(("", Command::Font(Font::C))));
        assert_eq!(Command::parse("flip true"), Ok(("", Command::Flip(true))));
        assert_eq!(Command::parse("flip false"), Ok(("", Command::Flip(false))));
        assert_eq!(
            Command::parse("justify left"),
            Ok(("", Command::Justify(JustifyMode::LEFT)))
        );
        assert_eq!(
            Command::parse("justify center"),
            Ok(("", Command::Justify(JustifyMode::CENTER)))
        );
        assert_eq!(
            Command::parse("justify right"),
            Ok(("", Command::Justify(JustifyMode::RIGHT)))
        );
        assert_eq!(Command::parse("feed 1"), Ok(("", Command::Feed(1))));
        assert_eq!(Command::parse("feed 128"), Ok(("", Command::Feed(128))));
        assert_eq!(Command::parse("feed"), Ok(("", Command::Feed(1))));
        assert_eq!(
            Command::parse("ean13 1234567890123"),
            Ok(("", Command::Ean13(String::from("1234567890123"))))
        );
        assert_eq!(
            Command::parse("ean8 12345678"),
            Ok(("", Command::Ean8(String::from("12345678"))))
        );
        assert_eq!(
            Command::parse("qr_code \"rofl.de\""),
            Ok(("", Command::QrCode(String::from("rofl.de"))))
        );

        let string = "write \"asdf\"\n     \twriteln \"rofl\"\ncut";
        let command = Program::parse(&string);
        assert_eq!(
            command,
            Ok((
                "",
                Program {
                    commands: vec![
                        Command::Write(String::from("asdf")),
                        Command::Write(String::from("rofl\n")),
                        Command::Cut
                    ]
                }
            ))
        );
    }
}

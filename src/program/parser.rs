use crate::printer;
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

use crate::program::{Command, Program};

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
                |string| Command::Raw(printer::Command::Write(string)),
            ),
            map(
                preceded(pair(tag("writeln"), space1), escaped_string),
                |string| Command::Raw(printer::Command::Write(string + "\n")),
            ),
            map(preceded(pair(tag("bold"), space1), bool), |mode| {
                Command::Raw(printer::Command::Bold(mode))
            }),
            map(
                preceded(pair(tag("underline"), space1), underline_mode),
                |mode| Command::Raw(printer::Command::Underline(mode)),
            ),
            map(preceded(pair(tag("double_strike"), space1), bool), |mode| {
                Command::Raw(printer::Command::DoubleStrike(mode))
            }),
            map(preceded(pair(tag("font"), space1), font), |font| {
                Command::Raw(printer::Command::Font(font))
            }),
            map(preceded(pair(tag("flip"), space1), bool), |flip| {
                Command::Raw(printer::Command::Flip(flip))
            }),
            map(
                preceded(pair(tag("justify"), space1), justify_mode),
                |mode| Command::Raw(printer::Command::Justify(mode)),
            ),
            map(preceded(pair(tag("reverse"), space1), bool), |reverse| {
                Command::Raw(printer::Command::Reverse(reverse))
            }),
            map(preceded(pair(tag("feed"), space1), u8), |lines| {
                Command::Raw(printer::Command::Feed(lines))
            }),
            map(tag("feed"), |_| Command::Raw(printer::Command::Feed(1))),
            map(
                preceded(
                    pair(tag("ean13"), space1),
                    take_while_m_n(12, 13, AsChar::is_dec_digit),
                ),
                |digits| Command::Raw(printer::Command::Ean13(String::from(digits))),
            ),
            map(
                preceded(
                    pair(tag("ean8"), space1),
                    take_while_m_n(7, 8, AsChar::is_dec_digit),
                ),
                |digits| Command::Raw(printer::Command::Ean8(String::from(digits))),
            ),
            map(
                preceded(pair(tag("qr_code"), space1), escaped_string),
                |data| Command::Raw(printer::Command::QrCode(data)),
            ),
            map(
                preceded(pair(tag("size"), space1), separated_pair(u8, tag(","), u8)),
                |(a, b)| Command::Raw(printer::Command::Size(a, b)),
            ),
            map(tag("reset_size"), |_| {
                Command::Raw(printer::Command::ResetSize)
            }),
            map(tag("sudoku"), |_| Command::Sudoku),
            map(tag("minicrossword"), |_| Command::MiniCrossword),
            map(tag("cut"), |_| Command::Raw(printer::Command::Cut)),
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
            Ok((
                "",
                Command::Raw(printer::Command::Write(String::from("rofl\n")))
            ))
        );
        assert_eq!(
            Command::parse("bold true"),
            Ok(("", Command::Raw(printer::Command::Bold(true))))
        );
        assert_eq!(
            Command::parse("bold false"),
            Ok(("", Command::Raw(printer::Command::Bold(false))))
        );
        assert_eq!(
            Command::parse("underline none"),
            Ok((
                "",
                Command::Raw(printer::Command::Underline(UnderlineMode::None))
            ))
        );
        assert_eq!(
            Command::parse("underline single"),
            Ok((
                "",
                Command::Raw(printer::Command::Underline(UnderlineMode::Single))
            ))
        );
        assert_eq!(
            Command::parse("underline double"),
            Ok((
                "",
                Command::Raw(printer::Command::Underline(UnderlineMode::Double))
            ))
        );
        assert_eq!(
            Command::parse("font a"),
            Ok(("", Command::Raw(printer::Command::Font(Font::A))))
        );
        assert_eq!(
            Command::parse("font b"),
            Ok(("", Command::Raw(printer::Command::Font(Font::B))))
        );
        assert_eq!(
            Command::parse("font c"),
            Ok(("", Command::Raw(printer::Command::Font(Font::C))))
        );
        assert_eq!(
            Command::parse("flip true"),
            Ok(("", Command::Raw(printer::Command::Flip(true))))
        );
        assert_eq!(
            Command::parse("flip false"),
            Ok(("", Command::Raw(printer::Command::Flip(false))))
        );
        assert_eq!(
            Command::parse("justify left"),
            Ok((
                "",
                Command::Raw(printer::Command::Justify(JustifyMode::LEFT))
            ))
        );
        assert_eq!(
            Command::parse("justify center"),
            Ok((
                "",
                Command::Raw(printer::Command::Justify(JustifyMode::CENTER))
            ))
        );
        assert_eq!(
            Command::parse("justify right"),
            Ok((
                "",
                Command::Raw(printer::Command::Justify(JustifyMode::RIGHT))
            ))
        );
        assert_eq!(
            Command::parse("feed 1"),
            Ok(("", Command::Raw(printer::Command::Feed(1))))
        );
        assert_eq!(
            Command::parse("feed 128"),
            Ok(("", Command::Raw(printer::Command::Feed(128))))
        );
        assert_eq!(
            Command::parse("feed"),
            Ok(("", Command::Raw(printer::Command::Feed(1))))
        );
        assert_eq!(
            Command::parse("ean13 1234567890123"),
            Ok((
                "",
                Command::Raw(printer::Command::Ean13(String::from("1234567890123")))
            ))
        );
        assert_eq!(
            Command::parse("ean8 12345678"),
            Ok((
                "",
                Command::Raw(printer::Command::Ean8(String::from("12345678")))
            ))
        );
        assert_eq!(
            Command::parse("qr_code \"rofl.de\""),
            Ok((
                "",
                Command::Raw(printer::Command::QrCode(String::from("rofl.de")))
            ))
        );

        let string = "write \"asdf\"\n     \twriteln \"rofl\"\ncut";
        let command = Program::parse(&string);
        assert_eq!(
            command,
            Ok((
                "",
                Program {
                    commands: vec![
                        Command::Raw(printer::Command::Write(String::from("asdf"))),
                        Command::Raw(printer::Command::Write(String::from("rofl\n"))),
                        Command::Raw(printer::Command::Cut)
                    ]
                }
            ))
        );
    }
}

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
        let (remains, commands) = many0(alt((
            // Parse a command line
            map(
                preceded(
                    space0,
                    terminated(Command::parse, preceded(space0, alt((eof, line_ending)))),
                ),
                Some,
            ),
            // Skip empty lines
            map(terminated(space0, line_ending), |_| None),
        )))
        .parse(input)?;

        Ok((
            remains,
            Program {
                commands: commands.into_iter().flatten().collect(),
            },
        ))
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
    pub fn parse(input: &str) -> IResult<&str, Command> {
        crate::documented_parser! {
            {
                name: "write",
                syntax: "write \"<text>\"",
                description: "Outputs text to the printer without a line break at the end",
                category: Text,
                examples: [
                    "write \"Hello World\"",
                    "write \"Price: $19.99\""
                ],
                parser: map(
                    preceded(pair(tag("write"), space1), escaped_string),
                    |string| Command::Raw(printer::Command::Write(string))
                )
            },
            {
                name: "writeln",
                syntax: "writeln \"<text>\"",
                description: "Outputs text to the printer followed by a line break",
                category: Text,
                examples: [
                    "writeln \"Hello World\"",
                    "writeln \"Order #12345\""
                ],
                parser: map(
                    preceded(pair(tag("writeln"), space1), escaped_string),
                    |string| Command::Raw(printer::Command::Write(string + "\n"))
                )
            },
            {
                name: "bold",
                syntax: "bold <true|false>",
                description: "Enables or disables bold text",
                category: Formatting,
                examples: [
                    "bold true",
                    "bold false"
                ],
                parser: map(
                    preceded(pair(tag("bold"), space1), bool),
                    |mode| Command::Raw(printer::Command::Bold(mode))
                )
            },
            {
                name: "underline",
                syntax: "underline <none|single|double>",
                description: "Sets the underline mode for text",
                category: Formatting,
                examples: [
                    "underline none",
                    "underline single",
                    "underline double"
                ],
                parser: map(
                    preceded(pair(tag("underline"), space1), underline_mode),
                    |mode| Command::Raw(printer::Command::Underline(mode))
                )
            },
            {
                name: "double_strike",
                syntax: "double_strike <true|false>",
                description: "Enables or disables double-strike for text",
                category: Formatting,
                examples: [
                    "double_strike true",
                    "double_strike false"
                ],
                parser: map(
                    preceded(pair(tag("double_strike"), space1), bool),
                    |mode| Command::Raw(printer::Command::DoubleStrike(mode))
                )
            },
            {
                name: "font",
                syntax: "font <a|b|c>",
                description: "Sets the font type. Available fonts depend on printer model and might fallback to another font if unavailable.",
                category: Formatting,
                examples: [
                    "font a",
                    "font b",
                    "font c"
                ],
                parser: map(
                    preceded(pair(tag("font"), space1), font),
                    |f| Command::Raw(printer::Command::Font(f))
                )
            },
            {
                name: "flip",
                syntax: "flip <true|false>",
                description: "Flips text 180 degrees",
                category: Formatting,
                examples: [
                    "flip true",
                    "flip false"
                ],
                parser: map(
                    preceded(pair(tag("flip"), space1), bool),
                    |flip| Command::Raw(printer::Command::Flip(flip))
                )
            },
            {
                name: "justify",
                syntax: "justify <left|center|right>",
                description: "Sets text justification/alignment",
                category: Layout,
                examples: [
                    "justify left",
                    "justify center",
                    "justify right"
                ],
                parser: map(
                    preceded(pair(tag("justify"), space1), justify_mode),
                    |mode| Command::Raw(printer::Command::Justify(mode))
                )
            },
            {
                name: "reverse",
                syntax: "reverse <true|false>",
                description: "Enables or disables inverted text colors (white text on black background)",
                category: Formatting,
                examples: [
                    "reverse true",
                    "reverse false"
                ],
                parser: map(
                    preceded(pair(tag("reverse"), space1), bool),
                    |reverse| Command::Raw(printer::Command::Reverse(reverse))
                )
            },
            {
                name: "feed",
                syntax: "feed <lines>",
                description: "Feeds paper forward by the specified number of lines",
                category: Layout,
                examples: [
                    "feed 1",
                    "feed 3",
                    "feed 10"
                ],
                parser: map(
                    preceded(pair(tag("feed"), space1), u8),
                    |lines| Command::Raw(printer::Command::Feed(lines))
                )
            },
            {
                name: "feed",
                syntax: "feed",
                description: "Feeds paper forward by 1 line (default)",
                category: Layout,
                examples: ["feed"],
                parser: map(tag("feed"), |_| Command::Raw(printer::Command::Feed(1)))
            },
            {
                name: "ean13",
                syntax: "ean13 <12-13 digits>",
                description: "Prints an EAN-13 barcode (12 or 13 digits)",
                category: Barcodes,
                examples: [
                    "ean13 1234567890123",
                    "ean13 123456789012"
                ],
                parser: map(
                    preceded(
                        pair(tag("ean13"), space1),
                        take_while_m_n(12, 13, AsChar::is_dec_digit)
                    ),
                    |digits| Command::Raw(printer::Command::Ean13(String::from(digits)))
                )
            },
            {
                name: "ean8",
                syntax: "ean8 <7-8 digits>",
                description: "Prints an EAN-8 barcode (7 or 8 digits)",
                category: Barcodes,
                examples: [
                    "ean8 12345678",
                    "ean8 1234567"
                ],
                parser: map(
                    preceded(
                        pair(tag("ean8"), space1),
                        take_while_m_n(7, 8, AsChar::is_dec_digit)
                    ),
                    |digits| Command::Raw(printer::Command::Ean8(String::from(digits)))
                )
            },
            {
                name: "qr_code",
                syntax: "qr_code \"<data>\"",
                description: "Prints a QR code with the specified data",
                category: Barcodes,
                examples: [
                    "qr_code \"https://example.com\"",
                    "qr_code \"Hello World\""
                ],
                parser: map(
                    preceded(pair(tag("qr_code"), space1), escaped_string),
                    |data| Command::Raw(printer::Command::QrCode(data))
                )
            },
            {
                name: "size",
                syntax: "size <width>,<height>",
                description: "Sets character size multiplier (1-8 for both width and height)",
                category: Formatting,
                examples: [
                    "size 1,1",
                    "size 2,2",
                    "size 3,1"
                ],
                parser: map(
                    preceded(pair(tag("size"), space1), separated_pair(u8, tag(","), u8)),
                    |(a, b)| Command::Raw(printer::Command::Size(a, b))
                )
            },
            {
                name: "reset_size",
                syntax: "reset_size",
                description: "Resets text size to default (1,1)",
                category: Formatting,
                examples: ["reset_size"],
                parser: map(tag("reset_size"), |_| Command::Raw(printer::Command::ResetSize))
            },
            {
                name: "sudoku",
                syntax: "sudoku",
                description: "Generates and prints a random Sudoku puzzle",
                category: Special,
                examples: ["sudoku"],
                parser: map(tag("sudoku"), |_| Command::Sudoku)
            },
            {
                name: "minicrossword",
                syntax: "minicrossword",
                description: "Generates and prints a mini crossword puzzle",
                category: Special,
                examples: ["minicrossword"],
                parser: map(tag("minicrossword"), |_| Command::MiniCrossword)
            },
            {
                name: "cut",
                syntax: "cut",
                description: "Cuts the paper (if printer has auto-cutter)",
                category: Special,
                examples: ["cut"],
                parser: map(tag("cut"), |_| Command::Raw(printer::Command::Cut))
            },
            {
                name: "todo",
                syntax: "todo \"<task>\"",
                description: "Adds a line rendered as a todo item",
                category: Special,
                examples: [
                    "todo \"Buy groceries\"",
                    "todo \"Call dentist\""
                ],
                parser: map(
                    preceded(pair(tag("todo"), space1), escaped_string),
                    Command::ToDo
                )
            }
        }
        .parse(input)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        let string = "write \"asdf\"\t\n   \n\n\t\n  \twriteln \"rofl\"\ncut";
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

    #[test]
    fn test_all_documented_examples_parse() {
        // Get all commands from the documentation registry
        use crate::program::doc_macros::get_registered_commands;

        // Ensure parser is loaded to register commands
        let _ = Command::parse("");

        let commands = get_registered_commands();
        assert!(
            !commands.is_empty(),
            "No commands registered - parser may not be loaded"
        );

        let mut total_examples = 0;
        let mut failed_examples = Vec::new();
        let num_commands = commands.len();

        for cmd in &commands {
            for example in &cmd.examples {
                total_examples += 1;

                // Try to parse the example
                match Command::parse(example) {
                    Ok((remaining, _parsed_cmd)) => {
                        // Ensure the entire input was consumed
                        if !remaining.is_empty() {
                            failed_examples.push(format!(
                                "Command '{}' example '{}' left unparsed input: '{}'",
                                cmd.name, example, remaining
                            ));
                        }
                    }
                    Err(e) => {
                        failed_examples.push(format!(
                            "Command '{}' example '{}' failed to parse: {:?}",
                            cmd.name, example, e
                        ));
                    }
                }
            }
        }

        if !failed_examples.is_empty() {
            panic!(
                "Failed to parse {} out of {} examples:\n{}",
                failed_examples.len(),
                total_examples,
                failed_examples.join("\n")
            );
        }

        println!(
            "✓ Successfully parsed all {} examples from {} commands",
            total_examples, num_commands
        );
    }
}

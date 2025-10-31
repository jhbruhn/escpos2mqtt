/// Integration test for macro-based documentation
use escpos2mqtt::program::doc_macros;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::IResult;
use nom::Parser;

#[derive(Debug, PartialEq)]
enum TestCommand {
    Foo,
    Bar,
}

fn test_parser(input: &str) -> IResult<&str, TestCommand> {
    escpos2mqtt::documented_parser! {
        {
            name: "foo",
            syntax: "foo",
            description: "Test foo command",
            category: Special,
            examples: ["foo"],
            parser: map(tag("foo"), |_| TestCommand::Foo)
        },
        {
            name: "bar",
            syntax: "bar",
            description: "Test bar command",
            category: Special,
            examples: ["bar"],
            parser: map(tag("bar"), |_| TestCommand::Bar)
        }
    }
    .parse(input)
}

#[test]
fn test_documented_parser_works() {
    // Parser should work normally
    assert_eq!(test_parser("foo"), Ok(("", TestCommand::Foo)));
    assert_eq!(test_parser("bar"), Ok(("", TestCommand::Bar)));
}

#[test]
fn test_documentation_registered() {
    // Trigger parser to ensure registration
    let _ = test_parser("foo");

    // Get registered commands
    let commands = doc_macros::get_registered_commands();

    // Should have some commands registered
    assert!(!commands.is_empty(), "No commands registered");

    // Check if our test commands are present (they might be mixed with others)
    let has_foo = commands.iter().any(|cmd| cmd.name == "foo");
    let has_bar = commands.iter().any(|cmd| cmd.name == "bar");

    assert!(has_foo || has_bar, "Test commands not found in registry");
}

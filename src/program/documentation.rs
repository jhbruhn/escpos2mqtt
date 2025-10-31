/// Command documentation and metadata for DSL reference generation
use std::fmt::Write;

/// Metadata for a DSL command
#[derive(Debug, Clone)]
pub struct CommandDoc {
    pub name: &'static str,
    pub syntax: &'static str,
    pub description: &'static str,
    pub examples: Vec<&'static str>,
    pub category: CommandCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Text,
    Formatting,
    Barcodes,
    Layout,
    Special,
}

impl CommandCategory {
    pub fn name(&self) -> &str {
        match self {
            CommandCategory::Text => "Text Output",
            CommandCategory::Formatting => "Text Formatting",
            CommandCategory::Barcodes => "Barcodes & QR Codes",
            CommandCategory::Layout => "Layout & Spacing",
            CommandCategory::Special => "Special Commands",
        }
    }
}

/// Get all documented commands from the macro registry
/// This is populated when the parser is loaded
pub fn all_commands() -> Vec<CommandDoc> {
    use super::doc_macros::get_registered_commands;
    get_registered_commands()
}


/// Generate markdown documentation for all commands
pub fn generate_markdown() -> String {
    generate_markdown_with_commands(all_commands())
}

/// Generate markdown documentation with specific commands
fn generate_markdown_with_commands(commands: Vec<CommandDoc>) -> String {
    let mut output = String::new();

    writeln!(output, "# ESC/POS DSL Reference").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "This document describes the Domain Specific Language (DSL) used to send printing commands to ESC/POS-compatible printers.").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "## Overview").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "The DSL consists of commands that are executed sequentially. Each command must be on its own line.").unwrap();
    writeln!(output, "Empty lines are ignored. String arguments must be enclosed in double quotes.").unwrap();
    writeln!(output).unwrap();

    // Group commands by category
    let categories: Vec<CommandCategory> = vec![
        CommandCategory::Text,
        CommandCategory::Formatting,
        CommandCategory::Layout,
        CommandCategory::Barcodes,
        CommandCategory::Special,
    ];

    for category in categories.iter() {
        writeln!(output, "## {}", category.name()).unwrap();
        writeln!(output).unwrap();

        let category_commands: Vec<_> = commands.iter()
            .filter(|cmd| &cmd.category == category)
            .collect();

        for cmd in category_commands {
            writeln!(output, "### `{}`", cmd.name).unwrap();
            writeln!(output).unwrap();
            writeln!(output, "**Syntax:** `{}`", cmd.syntax).unwrap();
            writeln!(output).unwrap();
            writeln!(output, "{}", cmd.description).unwrap();
            writeln!(output).unwrap();

            if !cmd.examples.is_empty() {
                writeln!(output, "**Examples:**").unwrap();
                writeln!(output).unwrap();
                writeln!(output, "```").unwrap();
                for example in &cmd.examples {
                    writeln!(output, "{}", example).unwrap();
                }
                writeln!(output, "```").unwrap();
                writeln!(output).unwrap();
            }
        }
    }

    writeln!(output, "## Complete Example").unwrap();
    writeln!(output).unwrap();
    writeln!(output, "```").unwrap();
    writeln!(output, "justify center").unwrap();
    writeln!(output, "bold true").unwrap();
    writeln!(output, "size 2,2").unwrap();
    writeln!(output, "writeln \"RECEIPT\"").unwrap();
    writeln!(output, "reset_size").unwrap();
    writeln!(output, "bold false").unwrap();
    writeln!(output, "feed 1").unwrap();
    writeln!(output, "justify left").unwrap();
    writeln!(output, "writeln \"Item 1          $10.00\"").unwrap();
    writeln!(output, "writeln \"Item 2          $15.00\"").unwrap();
    writeln!(output, "underline single").unwrap();
    writeln!(output, "writeln \"Total:          $25.00\"").unwrap();
    writeln!(output, "underline none").unwrap();
    writeln!(output, "feed 2").unwrap();
    writeln!(output, "justify center").unwrap();
    writeln!(output, "qr_code \"https://example.com/receipt/12345\"").unwrap();
    writeln!(output, "feed 2").unwrap();
    writeln!(output, "cut").unwrap();
    writeln!(output, "```").unwrap();

    output
}

/// Generate a command reference in plain text format
pub fn generate_text() -> String {
    generate_text_with_commands(all_commands())
}

/// Generate a command reference in plain text format with specific commands
fn generate_text_with_commands(commands: Vec<CommandDoc>) -> String {
    let mut output = String::new();

    writeln!(output, "ESC/POS DSL COMMAND REFERENCE").unwrap();
    writeln!(output, "==============================").unwrap();
    writeln!(output).unwrap();

    for cmd in commands {
        writeln!(output, "{}", cmd.name.to_uppercase()).unwrap();
        writeln!(output, "  Syntax: {}", cmd.syntax).unwrap();
        writeln!(output, "  {}", cmd.description).unwrap();
        if !cmd.examples.is_empty() {
            writeln!(output, "  Examples:").unwrap();
            for example in &cmd.examples {
                writeln!(output, "    {}", example).unwrap();
            }
        }
        writeln!(output).unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::Command;

    // Helper to ensure parser is loaded before tests
    fn ensure_parser_loaded() {
        let _ = Command::parse("");
    }

    #[test]
    fn test_all_commands_not_empty() {
        ensure_parser_loaded();
        let commands = all_commands();
        assert!(!commands.is_empty(), "No commands were registered. Make sure parser is loaded.");
    }

    #[test]
    fn test_generate_markdown() {
        ensure_parser_loaded();
        let markdown = generate_markdown();
        assert!(markdown.contains("# ESC/POS DSL Reference"));
        assert!(markdown.contains("write"));
        assert!(markdown.contains("bold"));
        assert!(markdown.contains("qr_code"));
    }

    #[test]
    fn test_generate_text() {
        ensure_parser_loaded();
        let text = generate_text();
        assert!(text.contains("ESC/POS DSL COMMAND REFERENCE"));
        assert!(text.contains("WRITE"));
        assert!(text.contains("BOLD"));
    }

    #[test]
    fn test_all_categories_present() {
        ensure_parser_loaded();
        let commands = all_commands();
        let categories: std::collections::HashSet<_> = commands
            .iter()
            .map(|cmd| &cmd.category)
            .collect();

        assert!(categories.contains(&CommandCategory::Text));
        assert!(categories.contains(&CommandCategory::Formatting));
        assert!(categories.contains(&CommandCategory::Barcodes));
        assert!(categories.contains(&CommandCategory::Layout));
        assert!(categories.contains(&CommandCategory::Special));
    }
}

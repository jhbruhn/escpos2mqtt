/// Macros for documenting DSL commands directly in the parser
/// This allows documentation to be co-located with the parser implementation

/// Storage for command documentation extracted from parser
use once_cell::sync::Lazy;
use std::sync::Mutex;

use super::documentation::CommandDoc;

/// Global registry of documented commands
pub static COMMAND_REGISTRY: Lazy<Mutex<Vec<CommandDoc>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Register a command in the documentation registry
pub fn register_command(doc: CommandDoc) {
    COMMAND_REGISTRY.lock().unwrap().push(doc);
}

/// Get all registered commands
pub fn get_registered_commands() -> Vec<CommandDoc> {
    COMMAND_REGISTRY.lock().unwrap().clone()
}

/// Macro to document a parser command
///
/// Usage:
/// ```ignore
/// doc_command! {
///     name: "write",
///     syntax: "write \"<text>\"",
///     description: "Outputs text to the printer without a line break",
///     category: Text,
///     examples: [
///         "write \"Hello World\"",
///         "write \"Price: $19.99\""
///     ],
///     parser: |input| {
///         map(
///             preceded(pair(tag("write"), space1), escaped_string),
///             |string| Command::Raw(printer::Command::Write(string)),
///         )
///         .parse(input)
///     }
/// }
/// ```
#[macro_export]
macro_rules! doc_command {
    (
        name: $name:expr,
        syntax: $syntax:expr,
        description: $description:expr,
        category: $category:ident,
        examples: [ $($example:expr),* $(,)? ],
        parser: $parser:expr
    ) => {{
        // Register the documentation
        #[allow(unused_imports)]
        use $crate::program::doc_macros::register_command;
        #[allow(unused_imports)]
        use $crate::program::documentation::{CommandDoc, CommandCategory};

        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            register_command(CommandDoc {
                name: $name,
                syntax: $syntax,
                description: $description,
                category: CommandCategory::$category,
                examples: vec![$($example),*],
            });
        });

        // Return the parser
        $parser
    }};
}

/// Helper macro to create a documented command parser function
#[macro_export]
macro_rules! documented_parser {
    (
        $(
            {
                name: $name:expr,
                syntax: $syntax:expr,
                description: $description:expr,
                category: $category:ident,
                examples: [ $($example:expr),* $(,)? ],
                parser: $parser:expr
            }
        ),* $(,)?
    ) => {{
        // Register all commands
        $({
            #[allow(unused_imports)]
            use $crate::program::doc_macros::register_command;
            #[allow(unused_imports)]
            use $crate::program::documentation::{CommandDoc, CommandCategory};

            static INIT: std::sync::Once = std::sync::Once::new();
            INIT.call_once(|| {
                register_command(CommandDoc {
                    name: $name,
                    syntax: $syntax,
                    description: $description,
                    category: CommandCategory::$category,
                    examples: vec![$($example),*],
                });
            });
        })*

        // Return combined parser
        alt((
            $($parser),*
        ))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::documentation::CommandCategory;

    #[test]
    fn test_command_doc_structure() {
        // Test that CommandDoc can be created with all fields
        let doc = CommandDoc {
            name: "example",
            syntax: "example <arg>",
            description: "An example command",
            category: CommandCategory::Text,
            examples: vec!["example test", "example foo"],
        };

        assert_eq!(doc.name, "example");
        assert_eq!(doc.syntax, "example <arg>");
        assert_eq!(doc.description, "An example command");
        assert_eq!(doc.category, CommandCategory::Text);
        assert_eq!(doc.examples.len(), 2);
        assert_eq!(doc.examples[0], "example test");
    }

    #[test]
    fn test_registry_functions_exist() {
        // Test that we can call the registry functions
        // Note: We don't register fake commands here because that would
        // interfere with the test_all_documented_examples_parse test
        // which expects all registered commands to be parseable.

        // Just verify we can get commands (will include real parser commands)
        let commands = get_registered_commands();

        // If parser has been loaded, we should have commands
        // Otherwise this just tests that the function doesn't panic
        let _ = commands.len();
    }
}

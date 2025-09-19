use crate::printer;
use crate::program::{Command, Program};
mod mini_crossword;
mod sudoku;

impl From<&Command> for Vec<printer::Command> {
    fn from(command: &Command) -> Vec<printer::Command> {
        match command {
            Command::Raw(cmd) => vec![cmd.clone()],
            Command::Sudoku => sudoku::make_sudoku(),
            Command::MiniCrossword => mini_crossword::make_mini_crossword(),
        }
    }
}

impl From<Program> for printer::Program {
    fn from(program: Program) -> printer::Program {
        printer::Program(
            program
                .commands
                .iter()
                .flat_map(|c| <&Command as Into<Vec<printer::Command>>>::into(c))
                .collect(),
        )
    }
}

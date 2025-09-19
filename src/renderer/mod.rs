use crate::printer;
use crate::program::{Command, Program};
mod mini_crossword;
mod sudoku;

pub async fn render(program: Program) -> printer::Program {
    printer::Program(
        futures::future::join_all(program.commands.iter().map(async |command| match command {
            Command::Raw(cmd) => vec![cmd.clone()],
            Command::Sudoku => sudoku::make_sudoku().await,
            Command::MiniCrossword => mini_crossword::make_mini_crossword().await,
        }))
        .await
        .iter()
        .flat_map(|f| f.clone())
        .collect(),
    )
}

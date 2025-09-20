use crate::printer;
use crate::program::{Command, Program};
mod mini_crossword;
mod sudoku;

pub struct RenderOptions {
    pub dpi: f32,
    pub chars_per_line: u32,
    pub pixels_per_char: u32,
}

pub async fn render(program: Program, render_options: RenderOptions) -> printer::Program {
    printer::Program(
        futures::future::join_all(program.commands.iter().map(async |command| match command {
            Command::Raw(cmd) => vec![cmd.clone()],
            Command::Sudoku => sudoku::make_sudoku().await,
            Command::MiniCrossword => mini_crossword::make_mini_crossword(&render_options).await,
            Command::ToDo(item) => {
                let prefix = "- [ ] ";
                textwrap::wrap(
                    item,
                    textwrap::Options::new(render_options.chars_per_line as usize)
                        .initial_indent(prefix)
                        .subsequent_indent(&" ".repeat(prefix.len())),
                )
                .into_iter()
                .map(|line| printer::Command::Write(format!("{}\n", line)))
                .collect()
            }
        }))
        .await
        .iter()
        .flat_map(|f| f.clone())
        .collect(),
    )
}

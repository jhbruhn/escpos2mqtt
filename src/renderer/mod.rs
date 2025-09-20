use crate::printer;
use crate::program::{Command, Program};
mod mini_crossword;
mod sudoku;

const DEFAULT_DPI: u16 = 180;
const DEFAULT_COLUMNS_PER_LINE: u8 = 42;
const DEFAULT_PIXELS_PER_LINE: u16 = 512;

pub async fn render<'a>(program: Program, profile: &escpos_db::Profile<'a>) -> printer::Program {
    let columns_per_line = profile
        .fonts
        .get(0) // todo: use currently active font
        .and_then(|font| Some(font.columns))
        .unwrap_or(DEFAULT_COLUMNS_PER_LINE);

    let dpi = profile.media.dpi.unwrap_or(DEFAULT_DPI);
    let width_px = profile
        .media
        .width
        .as_ref()
        .and_then(|m| Some(m.px))
        .unwrap_or(DEFAULT_PIXELS_PER_LINE);

    printer::Program(
        futures::future::join_all(program.commands.iter().map(async |command| {
            match command {
                Command::Raw(cmd) => vec![cmd.clone()],
                Command::Sudoku => sudoku::make_sudoku().await,
                Command::MiniCrossword => {
                    mini_crossword::make_mini_crossword(
                        dpi,
                        width_px as u32,
                        columns_per_line as u32,
                    )
                    .await
                }
                Command::ToDo(item) => {
                    let prefix = "- [ ] ";
                    std::iter::once(printer::Command::Justify(escpos::utils::JustifyMode::LEFT))
                        .chain(
                            textwrap::wrap(
                                item,
                                textwrap::Options::new(columns_per_line as usize)
                                    .initial_indent(prefix)
                                    .subsequent_indent(&" ".repeat(prefix.len())),
                            )
                            .into_iter()
                            .map(|line| printer::Command::Write(format!("{}\n", line))),
                        )
                        .collect()
                }
            }
        }))
        .await
        .iter()
        .flat_map(|f| f.clone())
        .collect(),
    )
}

use crate::printer;

mod parser;
pub mod documentation;
pub mod doc_macros;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Raw(printer::Command),
    Sudoku,
    MiniCrossword,
    ToDo(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub commands: Vec<Command>,
}

use jiff::civil::Date;
use resvg::{tiny_skia, usvg};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("request failed")]
    Request(#[from] reqwest::Error),
    #[error("SVG parsing failed")]
    SVG(#[from] usvg::Error),
    #[error("PNG parsing failed")]
    PNGEncoding,
    #[error("Ascii representation can only be rendered for 5x5 crosswords")]
    NotFiveByFive,
}

// This code is large derived from previous work by github.com/coolreader18

const NYT_MINI_CROSSWORD_URL: &str = "https://www.nytimes.com/svc/crosswords/v6/puzzle/mini.json";

pub struct MiniCrosswordOptions {
    pub dpi: u16,
    pub target_width: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiniCrossword {
    pub body: Vec<Puzzle>,
    constructors: Vec<String>,
    publication_date: Date,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Puzzle {
    board: String,
    pub clue_lists: Vec<ClueList>,
    pub clues: Vec<Clue>,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Cell {
    pub answer: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClueList {
    pub clues: Vec<u16>,
    pub name: Direction,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Direction {
    Across,
    Down,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Clue {
    // cells: Vec<u16>,
    // direction: Direction,
    pub label: String,
    pub text: Vec<ClueText>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClueText {
    pub plain: String,
    // formatted: Option<String>,
}

pub struct Crossword {
    pub image: Vec<u8>,
    pub puzzle: Puzzle,
    pub publication_date: Date,
    pub constructors: Vec<String>,
}

const CHARS: [[&str; 4]; 4] = [
    ["┌", "┬", "╥", "┐"], // top
    ["├", "┼", "╫", "┤"], // thin sep
    ["╞", "╪", "╬", "╡"], // thick sep
    ["└", "┴", "╨", "┘"], // bottom
];

impl Puzzle {
    /// Renders the 5x5 crossword puzzle as ASCII art
    pub fn render_ascii(&self) -> Result<String, Error> {
        if self.cells.len() != 5 * 5 {
            return Err(Error::NotFiveByFive);
        }
        let mut result = String::new();

        // Each cell is 8 characters wide (including borders)
        // Total width: 5 * 8 + 1 = 41 characters (under 42 limit)

        for row in 0..5 {
            // Top border of the row
            if row == 0 {
                result.push_str(&self.render_top_border());
            } else {
                result.push_str(&self.render_middle_border(row));
            }
            result.push('\n');

            // Cell content row
            result.push_str(&self.render_cell_row(row));
            result.push('\n');

            // Empty row for cell spacing
            result.push_str(&self.render_empty_row(row));
            result.push('\n');
        }

        // Bottom border
        result.push_str(&self.render_bottom_border());
        result.push('\n');

        Ok(result)
    }

    fn render_top_border(&self) -> String {
        let mut line = String::new();

        for col in 0..5 {
            let cell_idx = col;
            let is_black = self.cells[cell_idx].answer.is_none();

            if col == 0 {
                line.push_str(CHARS[0][0]); // ┌
            } else {
                line.push_str(CHARS[0][1]); // ┬
            }

            // Cell top border (7 chars)
            if is_black {
                line.push_str("███████");
            } else {
                line.push_str("───────");
            }
        }
        line.push_str(CHARS[0][3]); // ┐
        line
    }

    fn render_middle_border(&self, row: usize) -> String {
        let mut line = String::new();

        for col in 0..5 {
            let cell_idx = row * 5 + col;
            let above_idx = (row - 1) * 5 + col;

            let is_black = self.cells[cell_idx].answer.is_none();
            let above_black = self.cells[above_idx].answer.is_none();

            if col == 0 {
                line.push_str(CHARS[1][0]); // ├
            } else {
                line.push_str(CHARS[1][1]); // ┼
            }

            // Cell separator (7 chars)
            if is_black && above_black {
                line.push_str("███████");
            } else if is_black || above_black {
                line.push_str("───────");
            } else {
                line.push_str("───────");
            }
        }
        line.push_str(CHARS[1][3]); // ┤
        line
    }

    fn render_bottom_border(&self) -> String {
        let mut line = String::new();

        for col in 0..5 {
            if col == 0 {
                line.push_str(CHARS[3][0]); // └
            } else {
                line.push_str(CHARS[3][1]); // ┴
            }

            // Cell bottom border (7 chars)
            line.push_str("───────");
        }
        line.push_str(CHARS[3][3]); // ┘
        line
    }

    fn render_cell_row(&self, row: usize) -> String {
        let mut line = String::new();

        for col in 0..5 {
            let cell_idx = row * 5 + col;
            let cell = &self.cells[cell_idx];

            line.push('│'); // Left border

            if cell.answer.is_some() {
                // Cell has content
                if let Some(ref label) = cell.label {
                    // Cell with label (7 chars total)
                    line.push_str(&format!("{:<7}", label));
                } else {
                    // Cell without label (7 spaces)
                    line.push_str("       ");
                }
            } else {
                // Black cell (7 chars)
                line.push_str("███████");
            }
        }
        line.push('│'); // Right border
        line
    }

    fn render_empty_row(&self, row: usize) -> String {
        let mut line = String::new();

        for col in 0..5 {
            let cell_idx = row * 5 + col;
            let cell = &self.cells[cell_idx];

            line.push('│'); // Left border

            if cell.answer.is_some() {
                // Empty space in regular cell
                line.push_str("       ");
            } else {
                // Black cell continues
                line.push_str("███████");
            }
        }
        line.push('│'); // Right border
        line
    }
}

async fn fetch_mini_crossword() -> Result<MiniCrossword, reqwest::Error> {
    reqwest::get(NYT_MINI_CROSSWORD_URL).await?.json().await
}

fn render_crossword(svg: &str, options: MiniCrosswordOptions) -> Result<Vec<u8>, Error> {
    let mut opt = usvg::Options {
        dpi: options.dpi as f32,
        shape_rendering: usvg::ShapeRendering::CrispEdges,
        ..Default::default()
    };
    opt.fontdb_mut().load_system_fonts();
    let svg = usvg::Tree::from_str(&svg, &opt).map_err(Error::SVG)?;
    let size = svg.size();

    let scale = options.target_width as f32 / svg.size().width();
    let canvas_size = usvg::Size::from_wh(options.target_width as f32, size.height() * scale)
        .unwrap()
        .to_int_size();
    let trans = usvg::Transform::from_scale(scale, scale);

    let mut buf = tiny_skia::Pixmap::new(canvas_size.width(), canvas_size.height()).unwrap();
    resvg::render(&svg, trans, &mut buf.as_mut());
    buf.encode_png().map_err(|_| Error::PNGEncoding)
}

pub async fn get(options: MiniCrosswordOptions) -> Result<Crossword, Error> {
    let mini: MiniCrossword = fetch_mini_crossword().await.map_err(Error::Request)?;

    let puzzle = mini.body[0].clone();
    let png = render_crossword(&puzzle.board, options)?;

    Ok(Crossword {
        image: png,
        puzzle: puzzle,
        constructors: mini.constructors,
        publication_date: mini.publication_date,
    })
}

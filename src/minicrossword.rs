use jiff::civil::Date;
use resvg::{tiny_skia, usvg};
use serde::Deserialize;

// This code is large derived from previous work by github.com/coolreader18

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MiniCrossword {
    body: Vec<Puzzle>,
    constructors: Vec<String>,
    _editor: String,
    publication_date: Date,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Puzzle {
    board: String,
    pub clue_lists: Vec<ClueList>,
    pub clues: Vec<Clue>,
}

#[derive(Deserialize, Clone)]
pub struct ClueList {
    pub clues: Vec<u16>,
    pub name: Direction,
}

#[derive(Deserialize, Debug, Clone)]
pub enum Direction {
    Across,
    Down,
}

#[derive(Deserialize, Clone)]
pub struct Clue {
    // cells: Vec<u16>,
    // direction: Direction,
    pub label: String,
    pub text: Vec<ClueText>,
}

#[derive(Deserialize, Clone)]
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

pub fn get() -> anyhow::Result<Crossword> {
    const DPI: f32 = 203.0;

    let chars_per_line = 32.0;
    let pixels_per_char = 12.0;
    let mini: MiniCrossword =
        ureq::get("https://www.nytimes.com/svc/crosswords/v6/puzzle/mini.json")
            .header("User-Agent", "miniprinter")
            .call()?
            .body_mut()
            .read_json()?;

    let puzzle = mini.body[0].clone();

    let mut opt = usvg::Options {
        dpi: DPI,
        shape_rendering: usvg::ShapeRendering::CrispEdges,
        ..Default::default()
    };
    opt.fontdb_mut().load_system_fonts();
    let svg = usvg::Tree::from_str(&puzzle.board, &opt)?;
    let size = svg.size();

    let target_width = pixels_per_char * chars_per_line;

    let scale = target_width as f32 / svg.size().width();
    // canvas width should be the full width of the paper, but the render transform is rounded
    // to an even-ish number so that lines don't get lost rendering to the low resolution
    let scale = (scale * 100.0).round() / 100.0;
    let canvas_size = usvg::Size::from_wh(target_width, size.height() * scale)
        .unwrap()
        .to_int_size();
    let trans = usvg::Transform::from_scale(scale, scale);

    let mut buf = tiny_skia::Pixmap::new(canvas_size.width(), canvas_size.height()).unwrap();
    resvg::render(&svg, trans, &mut buf.as_mut());
    let png = buf.encode_png()?;

    Ok(Crossword {
        image: png,
        puzzle: puzzle,
        constructors: mini.constructors,
        publication_date: mini.publication_date,
    })
}

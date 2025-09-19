use jiff::civil::Date;
use resvg::{tiny_skia, usvg};
use serde::Deserialize;

#[derive(Debug)]
pub enum Error {
    Request(reqwest::Error),
    SVG(usvg::Error),
    PNGEncoding,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MiniCrossword Error: ")?;
        match self {
            Error::Request(re) => write!(f, "{re}"),
            Error::SVG(se) => write!(f, "{se}"),
            Error::PNGEncoding => write!(f, "PNG Encoding"),
        }
    }
}

impl std::error::Error for Error {}

// This code is large derived from previous work by github.com/coolreader18

const NYT_MINI_CROSSWORD_URL: &str = "https://www.nytimes.com/svc/crosswords/v6/puzzle/mini.json";

pub struct MiniCrosswordOptions {
    pub dpi: f32,
    pub chars_per_line: u8,
    pub pixels_per_char: u8,
}

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

async fn fetch_mini_crossword() -> Result<MiniCrossword, reqwest::Error> {
    reqwest::get(NYT_MINI_CROSSWORD_URL).await?.json().await
}

fn render_crossword(svg: &str, options: MiniCrosswordOptions) -> Result<Vec<u8>, Error> {
    let mut opt = usvg::Options {
        dpi: options.dpi,
        shape_rendering: usvg::ShapeRendering::CrispEdges,
        ..Default::default()
    };
    opt.fontdb_mut().load_system_fonts();
    let svg = usvg::Tree::from_str(&svg, &opt).map_err(Error::SVG)?;
    let size = svg.size();

    let target_width = options.pixels_per_char as u32 * options.chars_per_line as u32;

    let scale = target_width as f32 / svg.size().width();
    // canvas width should be the full width of the paper, but the render transform is rounded
    // to an even-ish number so that lines don't get lost rendering to the low resolution
    let scale = (scale * 100.0).round() / 100.0;
    let canvas_size = usvg::Size::from_wh(target_width as f32, size.height() * scale)
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

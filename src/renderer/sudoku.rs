use crate::printer::Command;
use escpos::utils::JustifyMode;
use rustoku_lib::generate_board;

fn render_sudoku(board: rustoku_lib::core::Board) -> String {
    let mut s = String::with_capacity(3 * 9 * 9 * 3);
    const CHARS: [[&str; 4]; 4] = [
        ["┌", "┬", "╥", "┐"], // top
        ["├", "┼", "╫", "┤"], // thin sep
        ["╞", "╪", "╬", "╡"], // thick sep
        ["└", "┴", "╨", "┘"], // bottom
    ];

    for row in 0..=9 {
        // Horizontal line
        if row == 0 || row == 9 {
            let chars = &CHARS[if row == 0 { 0 } else { 3 }];

            s.push_str(chars[0]);
            for i in 0..9 {
                s.push_str("───");
                if i < 8 {
                    s.push_str(chars[if i % 3 == 2 { 2 } else { 1 }]);
                }
            }
            s.push_str(chars[3]);
            s.push_str("\n");
        }

        if row < 9 {
            // Data row
            s.push_str("|");
            for col in 0..9 {
                let n = board.get(row, col);
                s.push_str(&format!(
                    " {} ",
                    if n > 0 {
                        n.to_string()
                    } else {
                        " ".to_string()
                    }
                ));
                s.push_str(if col % 3 == 2 && col < 8 {
                    "║"
                } else {
                    "│"
                });
            }
            s.push_str("\n");

            // Separator
            if row < 8 {
                let chars = &CHARS[if row % 3 == 2 { 2 } else { 1 }];
                s.push_str(chars[0]);
                for i in 0..9 {
                    s.push_str(if row % 3 == 2 {
                        "═══"
                    } else {
                        "───"
                    });
                    if i < 8 {
                        s.push_str(chars[if i % 3 == 2 { 2 } else { 1 }]);
                    }
                }
                s.push_str(chars[3]);
                s.push_str("\n");
            }
        }
    }

    s
}

pub async fn make_sudoku() -> Vec<Command> {
    let sudoku = generate_board(40).expect("sudokus should always be solvable.");
    vec![
        Command::ResetSize,
        Command::Justify(JustifyMode::CENTER),
        Command::Write(render_sudoku(sudoku)),
        Command::Justify(JustifyMode::LEFT),
        Command::ResetSize,
    ]
}

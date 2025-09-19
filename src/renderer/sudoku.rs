use crate::printer::Command;
use escpos::utils::JustifyMode;
use rustoku_lib::generate_board;

pub fn make_sudoku() -> Vec<Command> {
    let sudoku = generate_board(40).expect("sudokus should always be solvable.");
    let mut commands = vec![Command::ResetSize, Command::Justify(JustifyMode::CENTER)];

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

            commands.push(Command::Write(String::from(chars[0])));
            for i in 0..9 {
                commands.push(Command::Write(String::from("───")));
                if i < 8 {
                    commands.push(Command::Write(String::from(
                        chars[if i % 3 == 2 { 2 } else { 1 }],
                    )));
                }
            }
            commands.push(Command::Write(String::from(chars[3]) + "\n"));
        }

        if row < 9 {
            // Data row
            commands.push(Command::Write(String::from("|")));
            for col in 0..9 {
                let n = sudoku.get(row, col);
                commands.push(Command::Write(format!(
                    " {} ",
                    if n > 0 {
                        n.to_string()
                    } else {
                        " ".to_string()
                    }
                )));
                commands.push(Command::Write(String::from(if col % 3 == 2 && col < 8 {
                    "║"
                } else {
                    "│"
                })));
            }
            commands.push(Command::Write(String::from("\n")));

            // Separator
            if row < 8 {
                let chars = &CHARS[if row % 3 == 2 { 2 } else { 1 }];
                commands.push(Command::Write(String::from(chars[0])));
                for i in 0..9 {
                    commands.push(Command::Write(String::from(if row % 3 == 2 {
                        "═══"
                    } else {
                        "───"
                    })));
                    if i < 8 {
                        commands.push(Command::Write(String::from(
                            chars[if i % 3 == 2 { 2 } else { 1 }],
                        )));
                    }
                }
                commands.push(Command::Write(String::from(chars[3]) + "\n"));
            }
        }
    }
    commands.push(Command::Justify(JustifyMode::LEFT));
    commands.push(Command::ResetSize);

    commands
}

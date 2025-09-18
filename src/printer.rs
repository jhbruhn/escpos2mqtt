use crate::minicrossword;
use crate::program;
use escpos::driver::Driver;
use escpos::errors::Result;
use escpos::printer_options::PrinterOptions;
use escpos::utils::DebugMode;
use escpos::utils::Protocol;
use rustoku_lib::generate_board;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;
use unicode_width::UnicodeWidthStr;

struct Job(program::Program, Sender<Result<()>>);

pub struct Printer {
    program_sender: UnboundedSender<Job>,
}

impl Printer {
    pub fn new<D: Driver, F: Fn() -> Result<D> + Send + Sync + 'static>(driver_builder: F) -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Job>();

        tokio::spawn(async move {
            while !receiver.is_closed() {
                if let Some(Job(program, responder)) = receiver.recv().await {
                    let result = (|| {
                        let driver = (driver_builder)()?;
                        log::info!("Connected to printer.");

                        let mut printer = escpos::printer::Printer::new(
                            driver,
                            Protocol::default(),
                            Some(PrinterOptions::default()),
                        );
                        printer
                            .debug_mode(Some(DebugMode::Dec))
                            .init()?
                            .page_code(escpos::utils::PageCode::PC437)?
                            .smoothing(false)?;

                        for command in &program.commands {
                            use program::Command::*;
                            match command {
                                Write(text) => printer.write(text)?,
                                Bold(bold) => printer.bold(*bold)?,
                                Underline(mode) => printer.underline(*mode)?,
                                DoubleStrike(mode) => printer.double_strike(*mode)?,
                                Font(font) => printer.font(*font)?,
                                Flip(flip) => printer.flip(*flip)?,
                                Justify(mode) => printer.justify(*mode)?,
                                Reverse(reverse) => printer.reverse(*reverse)?,
                                Feed(lines) => printer.feeds(*lines)?,
                                Ean13(string) => printer.ean13(&string)?,
                                Ean8(string) => printer.ean8(&string)?,
                                QrCode(string) => printer.qrcode(&string)?,
                                Size(x, y) => printer.size(*x, *y)?,
                                ResetSize => printer.reset_size()?,
                                Sudoku => print_sudoku(&mut printer)?,
                                MiniCrossword => print_mini_crossword(&mut printer)?,
                                Cut => printer.cut()?,
                                //_ => &mut self.printer,
                            };
                        }

                        printer.print()?;
                        Ok(())
                    })();
                    responder
                        .send(result)
                        .expect("Response channel closed. This shouldn't happen.");
                }
            }
        });

        Self {
            program_sender: sender,
        }
    }

    pub async fn print(&mut self, program: program::Program) -> Result<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.program_sender
            .send(Job(program, sender))
            .expect("Job queue closed. This shouldn't happen.");
        receiver.await.unwrap()
    }
}

fn format_list(s: &[String]) -> String {
    match s {
        [x] => x.clone(),
        [x, y] => [x, " and ", y].concat(),
        xs => {
            let mut out = String::new();
            for (i, x) in xs.iter().enumerate() {
                if i == xs.len() - 1 {
                    out.push_str(", and ");
                } else if i != 0 {
                    out.push_str(", ");
                }
                out.push_str(x);
            }
            out
        }
    }
}

fn print_mini_crossword<D: Driver>(
    printer: &mut escpos::printer::Printer<D>,
) -> Result<&mut escpos::printer::Printer<D>> {
    let cw = minicrossword::get().expect("Could not get crossword");
    let puzzle = cw.puzzle;
    let wrap_opts = || textwrap::Options::new(42);

    printer.reset_size()?;
    printer
        .writeln(&cw.publication_date.strftime("%A, %B %-d, %Y").to_string())?
        .feed()?;
    printer.justify(escpos::utils::JustifyMode::CENTER)?;
    printer.bit_image_from_bytes(&cw.image)?;
    printer.feeds(2)?;
    printer.justify(escpos::utils::JustifyMode::LEFT)?;

    let write_wrapped =
        |printer: &mut escpos::printer::Printer<_>, text, opts: textwrap::Options<'_>| {
            let text = textwrap::wrap(text, opts);
            text.into_iter()
                .try_for_each(|line| printer.writeln(&line).map(drop))
        };

    for clues in &puzzle.clue_lists {
        printer.writeln(&format!("{:?}:", clues.name))?;
        for &clue_num in &clues.clues {
            let clue = &puzzle.clues[clue_num as usize];
            let label = format!("{}: ", clue.label);
            write_wrapped(
                printer,
                &clue.text[0].plain,
                wrap_opts()
                    .initial_indent(&label)
                    .subsequent_indent(&" ".repeat(label.width())),
            )?;
        }
        printer.feed()?;
    }

    write_wrapped(
        printer,
        &format_list(&cw.constructors),
        wrap_opts().initial_indent("By ").subsequent_indent("   "),
    )?;

    printer.reset_size()?;
    Ok(printer)
}

fn print_sudoku<D: Driver>(
    printer: &mut escpos::printer::Printer<D>,
) -> Result<&mut escpos::printer::Printer<D>> {
    let sudoku = generate_board(40).expect("sudokus should always be solvable.");

    printer.reset_size()?;
    printer.justify(escpos::utils::JustifyMode::CENTER)?;

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
            printer.write(chars[0])?;
            for i in 0..9 {
                printer.write("───")?;
                if i < 8 {
                    printer.write(chars[if i % 3 == 2 { 2 } else { 1 }])?;
                }
            }
            printer.writeln(chars[3])?;
        }

        if row < 9 {
            // Data row
            printer.write("│")?;
            for col in 0..9 {
                let n = sudoku.get(row, col);
                printer.write(&format!(
                    " {} ",
                    if n > 0 {
                        n.to_string()
                    } else {
                        " ".to_string()
                    }
                ))?;
                printer.write(if col % 3 == 2 && col < 8 {
                    "║"
                } else {
                    "│"
                })?;
            }
            printer.writeln("")?;

            // Separator
            if row < 8 {
                let chars = &CHARS[if row % 3 == 2 { 2 } else { 1 }];
                printer.write(chars[0])?;
                for i in 0..9 {
                    printer.write(if row % 3 == 2 {
                        "═══"
                    } else {
                        "───"
                    })?;
                    if i < 8 {
                        printer.write(chars[if i % 3 == 2 { 2 } else { 1 }])?;
                    }
                }
                printer.writeln(chars[3])?;
            }
        }
    }
    printer.justify(escpos::utils::JustifyMode::LEFT)?;
    printer.reset_size()
}

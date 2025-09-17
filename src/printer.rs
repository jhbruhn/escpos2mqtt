use crate::program;
use escpos::driver::Driver;
use escpos::errors::PrinterError;
use escpos::errors::Result;
use escpos::printer_options::PrinterOptions;
use escpos::utils::DebugMode;
use escpos::utils::Protocol;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;

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

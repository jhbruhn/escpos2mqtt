use escpos::driver::Driver;
use escpos::driver::NetworkDriver;
use escpos::errors::PrinterError;
use escpos::printer_options::PrinterOptions;
use escpos::utils::BitImageOption;
use escpos::utils::DebugMode;
use escpos::utils::Font;
use escpos::utils::JustifyMode;
use escpos::utils::Protocol;
use escpos::utils::UnderlineMode;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;

mod discover;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to print")]
    Printer(#[from] PrinterError),
    #[error("discovery error")]
    Discovery(#[from] discover::Error),
}

#[derive(Debug)]
pub struct Printer {
    pub name: String,
    pub description: String,
    program_sender: UnboundedSender<Job>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Write(String),
    Bold(bool),
    Underline(UnderlineMode),
    DoubleStrike(bool),
    Font(Font),
    Flip(bool),
    Justify(JustifyMode),
    Reverse(bool),
    Feed(u8),
    Ean13(String),
    Ean8(String),
    QrCode(String),
    Size(u8, u8),
    ResetSize,
    Cut,
    BitImageFromBytesWithWidth(Vec<u8>, u32),
}

pub struct Program(pub Vec<Command>);

enum Job {
    Print(Program, Sender<Result<(), Error>>),
    GetModelName(Sender<Result<String, Error>>),
}

impl Printer {
    pub fn new<D: Driver + Clone, F: Fn() -> Result<D, PrinterError> + Send + Sync + 'static>(
        driver_builder: F,
        name: &str,
        description: &str,
    ) -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Job>();

        tokio::spawn(async move {
            while !receiver.is_closed() {
                let next = receiver.recv().await;

                if let Some(job) = next {
                    if let Job::Print(program, responder) = job {
                        let result = (|| {
                            let driver = (driver_builder)()?;
                            let mut printer = escpos::printer::Printer::new(
                                driver,
                                Protocol::default(),
                                Some(PrinterOptions::default()),
                            );
                            log::info!("Connected to printer.");
                            printer.debug_mode(Some(DebugMode::Dec));
                            printer
                                .init()?
                                .page_code(escpos::utils::PageCode::PC437)?
                                .smoothing(false)?;
                            for command in &program.0 {
                                use Command::*;
                                match command {
                                    Write(text) => printer.write(&text)?,
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
                                    BitImageFromBytesWithWidth(bytes, width) => printer
                                        .bit_image_from_bytes_option(&bytes, {
                                            BitImageOption::new(
                                                Some(*width),
                                                None,
                                                escpos::utils::BitImageSize::Normal,
                                            )?
                                        })?, //_ => &mut self.printer,
                                };
                            }

                            printer.print()?;
                            Ok(())
                        })()
                        .map_err(Error::Printer);
                        responder
                            .send(result)
                            .expect("Response channel closed. This shouldn't happen.");
                    } else if let Job::GetModelName(sender) = job {
                        let result = (|| {
                            let driver = (driver_builder)()?;
                            driver.write(&[0x1D, 0x49, 67])?;
                            driver.flush()?;
                            let mut response = [0 as u8; 82];
                            driver.read(&mut response)?;
                            let string_end = &response[1..].iter().position(|x| *x == 0_u8);
                            Ok(
                                String::from_utf8_lossy(&response[1..=string_end.unwrap_or(80)])
                                    .into_owned(),
                            )
                        })()
                        .map_err(Error::Printer);
                        sender
                            .send(result)
                            .expect("Response channel closed. This shouldn't happen.");
                    }
                }
            }
        });

        Self {
            program_sender: sender,
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    pub async fn print(&mut self, program: Program) -> Result<(), Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.program_sender
            .send(Job::Print(program, sender))
            .expect("Job queue closed. This shouldn't happen.");
        receiver.await.unwrap()
    }

    pub async fn model_name(&mut self) -> Result<String, Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.program_sender
            .send(Job::GetModelName(sender))
            .expect("Job queue closed. This shouldn't happen.");
        receiver.await.unwrap()
    }
}

pub async fn discover_network() -> Result<Vec<Printer>, Error> {
    let printers = discover::discover_network_printers()
        .await
        .map_err(Error::Discovery)?;
    Ok(printers
        .into_iter()
        .map(|info| {
            Printer::new(
                move || {
                    NetworkDriver::open(&info.address.ip().to_string(), info.address.port(), None)
                },
                &info.name,
                &info.description,
            )
        })
        .collect())
}

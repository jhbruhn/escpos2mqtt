use escpos::driver::Driver;
use escpos::errors::Result;
use escpos::printer_options::PrinterOptions;
use escpos::utils::DebugMode;
use escpos::utils::Protocol;

use crate::program;

pub struct Printer<D: Driver, F: Fn() -> Result<D>> {
    driver_builder: F,
}

impl<D: Driver, F: Fn() -> Result<D>> Printer<D, F> {
    pub fn new(driver_builder: F) -> Self {
        Self {
            driver_builder: driver_builder,
        }
    }

    pub fn print(&mut self, program: &program::Program) -> Result<()> {
        let driver = (self.driver_builder)()?;
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
    }
}

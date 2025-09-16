use escpos::driver::Driver;
use escpos::printer_options::PrinterOptions;
use escpos::utils::DebugMode;
use escpos::utils::Protocol;

use crate::program;

pub struct Printer<D: Driver> {
    printer: escpos::printer::Printer<D>,
}

impl<D: Driver> Printer<D> {
    pub fn new(driver: D) -> Self {
        let printer = escpos::printer::Printer::new(
            driver,
            Protocol::default(),
            Some(PrinterOptions::default()),
        );
        Self { printer: printer }
    }

    pub fn print(
        &mut self,
        program: &program::Program,
    ) -> Result<(), escpos::errors::PrinterError> {
        self.printer
            .debug_mode(Some(DebugMode::Dec))
            .init()?
            .smoothing(true)?;

        for command in &program.commands {
            use program::Command::*;
            match command {
                Write(text) => self.printer.write(text)?,
                Bold(bold) => self.printer.bold(*bold)?,
                Underline(mode) => self.printer.underline(*mode)?,
                DoubleStrike(mode) => self.printer.double_strike(*mode)?,
                Font(font) => self.printer.font(*font)?,
                Flip(flip) => self.printer.flip(*flip)?,
                Justify(mode) => self.printer.justify(*mode)?,
                Reverse(reverse) => self.printer.reverse(*reverse)?,
                Feed(lines) => self.printer.feeds(*lines)?,
                Ean13(string) => self.printer.ean13(&string)?,
                Ean8(string) => self.printer.ean8(&string)?,
                QrCode(string) => self.printer.qrcode(&string)?,
                Cut => self.printer.cut()?,
                //_ => &mut self.printer,
            };
        }

        self.printer.print()?;

        Ok(())
    }
}

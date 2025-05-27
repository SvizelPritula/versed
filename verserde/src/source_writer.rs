use std::{
    fmt::Arguments,
    io::{Result, Write},
};

pub struct SourceWriter<Writer> {
    writer: Writer,
    indent_level: usize,
    line_started: bool,
}

impl<Writer: Write> SourceWriter<Writer> {
    const INDENT: &str = "    ";

    pub fn new(writer: Writer) -> SourceWriter<Writer> {
        SourceWriter {
            writer,
            indent_level: 0,
            line_started: false,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    fn try_start_line(&mut self) -> Result<()> {
        if !self.line_started {
            self.line_started = true;

            for _ in 0..self.indent_level {
                self.writer.write(Self::INDENT.as_bytes())?;
            }
        }

        Ok(())
    }

    pub fn nl(&mut self) -> Result<()> {
        self.writer.write(b"\n")?;
        self.line_started = false;
        Ok(())
    }

    pub fn write(&mut self, str: &str) -> Result<()> {
        self.try_start_line()?;
        self.writer.write(str.as_bytes())?;
        Ok(())
    }

    pub fn write_nl(&mut self, str: &str) -> Result<()> {
        self.try_start_line()?;
        self.writer.write(str.as_bytes())?;
        self.nl()
    }

    pub fn write_fmt(&mut self, fmt: Arguments) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_fmt(fmt)?;
        Ok(())
    }

    pub fn write_fmt_nl(&mut self, fmt: Arguments) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_fmt(fmt)?;
        self.nl()
    }
}

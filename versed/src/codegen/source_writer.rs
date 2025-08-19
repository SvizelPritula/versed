use std::{
    fmt::Arguments,
    io::{Result, Write},
};

pub struct SourceWriter<Writer> {
    writer: Writer,
    indent_level: usize,
    line_started: bool,
    blank_line: BlankLineState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlankLineState {
    Normal,
    Requested,
    Prohibited,
}

impl<Writer: Write> SourceWriter<Writer> {
    const INDENT: &str = "    ";

    pub fn new(writer: Writer) -> SourceWriter<Writer> {
        SourceWriter {
            writer,
            indent_level: 0,
            line_started: false,
            blank_line: BlankLineState::Prohibited,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
        self.blank_line = BlankLineState::Prohibited;
    }

    pub fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
        self.blank_line = BlankLineState::Normal;
    }

    fn try_start_line(&mut self) -> Result<()> {
        if !self.line_started {
            if self.blank_line == BlankLineState::Requested {
                self.nl()?;
            }
            self.blank_line = BlankLineState::Normal;

            self.line_started = true;

            for _ in 0..self.indent_level {
                self.writer.write_all(Self::INDENT.as_bytes())?;
            }
        }

        Ok(())
    }

    pub fn nl(&mut self) -> Result<()> {
        self.writer.write_all(b"\n")?;
        self.line_started = false;
        Ok(())
    }

    pub fn write(&mut self, str: &str) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_all(str.as_bytes())?;
        Ok(())
    }

    pub fn write_nl(&mut self, str: &str) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_all(str.as_bytes())?;
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

    pub fn blank_line(&mut self) {
        if self.blank_line != BlankLineState::Prohibited {
            self.blank_line = BlankLineState::Requested;
        }
    }

    pub fn into_inner(self) -> Writer {
        self.writer
    }
}

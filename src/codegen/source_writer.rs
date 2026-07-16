//! Defines [`SourceWriter`], a utility for writing source code.

use std::{
    fmt::Arguments,
    io::{Result, Write},
};

/// Wraps a [`Write`] stream while tracking the indentation level.
///
/// It also has support for inserting blank lines using [`SourceWriter::blank_line`].
/// To make the automatic indentation work correctly, it's vital that a newline character
/// is never written to the stream manually (i.e. by writing a string that contains it).
/// Instead, [`SourceWriter::nl`] should be used, along with
/// [`SourceWriter::write_nl`] and [`SourceWriter::write_fmt_nl`].
pub struct SourceWriter<Writer> {
    writer: Writer,
    indent_level: usize,
    line_started: bool,
    blank_line: BlankLineState,
}

/// The state of the blank line insertion state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlankLineState {
    /// No blank line has been requested.
    Normal,
    /// A blank line has been requested by calling [`SourceWriter::blank_line`].
    Requested,
    /// A blank line cannot appear here, calls to [`SourceWriter::blank_line`] do nothing.
    Prohibited,
}

impl<Writer: Write> SourceWriter<Writer> {
    /// One level of indentation.
    const INDENT: &str = "    ";

    /// Creates a new [`SourceWriter`] by wrapping a stream.
    pub fn new(writer: Writer) -> SourceWriter<Writer> {
        SourceWriter {
            writer,
            indent_level: 0,
            line_started: false,
            blank_line: BlankLineState::Prohibited,
        }
    }

    /// Increases indentation by one level.
    pub fn indent(&mut self) {
        self.indent_level += 1;
        self.blank_line = BlankLineState::Prohibited;
    }

    /// Decreases indentation by one level.
    pub fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
        self.blank_line = BlankLineState::Normal;
    }

    /// Starts a new line, if it hasn't been started yet.
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

    /// Finishes the current line.
    pub fn nl(&mut self) -> Result<()> {
        self.writer.write_all(b"\n")?;
        self.line_started = false;
        Ok(())
    }

    /// Appends a string to the current line.
    pub fn write(&mut self, str: &str) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_all(str.as_bytes())?;
        Ok(())
    }

    /// Appends a string to the current line, and finishes the current line.
    pub fn write_nl(&mut self, str: &str) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_all(str.as_bytes())?;
        self.nl()
    }

    /// Appends formatted content to the current line.
    pub fn write_fmt(&mut self, fmt: Arguments) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_fmt(fmt)?;
        Ok(())
    }

    /// Appends formatted content to the current line, and finishes the current line.
    pub fn write_fmt_nl(&mut self, fmt: Arguments) -> Result<()> {
        self.try_start_line()?;
        self.writer.write_fmt(fmt)?;
        self.nl()
    }

    /// Inserts a blank line.
    ///
    /// Blank lines will be deduplicated, only one may appear in a row.
    /// Additionally, blank lines will be suppressed immediately
    /// after calling [`SourceWriter::indent`] or before calling [`SourceWriter::dedent`].
    pub fn blank_line(&mut self) {
        if self.blank_line != BlankLineState::Prohibited {
            self.blank_line = BlankLineState::Requested;
        }
    }

    /// Unwraps the [`SourceWriter`], returning the contained stream.
    pub fn into_inner(self) -> Writer {
        self.writer
    }
}

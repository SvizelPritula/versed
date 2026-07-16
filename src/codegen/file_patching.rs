//! Utilities for modifying existing files.

use std::{
    cmp::Reverse,
    fmt,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Result, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
};

/// Appends an extension to a file path.
///
/// The path should point to a file and the extension should start with a dot.
pub fn add_extension(path: &Path, extension: &str) -> PathBuf {
    let mut filename = path
        .file_name()
        .expect("path to point to a file")
        .to_owned();
    filename.push(extension);
    path.with_file_name(filename)
}

/// Appends a line to the end of a given file.
///
/// Will add a trailing newline (LF) to the file first, if it didn't have one before.
pub fn add_line_to_file(path: &Path, content: fmt::Arguments) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open(path)?;

    let pos = file.seek(SeekFrom::End(0))?;

    let must_add_lf = if pos > 0 {
        file.seek_relative(-1)?;

        let mut byte_buf = [0];
        file.read_exact(&mut byte_buf)?;
        let [byte] = byte_buf;

        byte != b'\n'
    } else {
        false
    };

    let mut file = BufWriter::new(file);

    if must_add_lf {
        file.write_all(b"\n")?;
    }

    file.write_fmt(content)?;
    file.write_all(b"\n")?;
    file.flush()?;

    Ok(())
}

/// Writes enough newline characters to `file` to make `content` end with at least `count`.
fn add_newlines_if_needed(count: usize, content: &str, mut file: impl Write) -> Result<()> {
    let existing = content.chars().rev().take_while(|c| *c == '\n').count();
    let count = count.saturating_sub(existing);

    for _ in 0..count {
        file.write_all(b"\n")?;
    }

    Ok(())
}

/// Writes the concatenation of two strings to a file, adding newlines if needed.
pub fn concat_files(a: &str, b: &str, path: &Path) -> Result<()> {
    let mut file = BufWriter::new(File::create(path)?);

    file.write_all(a.as_bytes())?;
    add_newlines_if_needed(2, a, &mut file)?;
    file.write_all(b.as_bytes())?;
    add_newlines_if_needed(1, a, &mut file)?;

    file.flush()?;
    Ok(())
}

/// Text to be inserted into a file at a specified byte position.
#[derive(Debug)]
pub struct AddEdit {
    index: usize,
    content: String,
}

impl AddEdit {
    /// Creates a new edit.
    pub fn new(index: usize, content: String) -> AddEdit {
        AddEdit { index, content }
    }
}

/// Applies a list of [`AddEdit`]s to a file.
pub fn apply_add_edits<W: Write>(file: &mut W, src: &str, mut edits: Vec<AddEdit>) -> Result<()> {
    edits.sort_by_key(|e| Reverse(e.index));

    for (index, byte) in src.bytes().enumerate() {
        while let Some(edit) = edits.pop_if(|e| e.index == index) {
            file.write_all(edit.content.as_bytes())?;
        }

        file.write_all(&[byte])?;
    }

    Ok(())
}

/// A range of byte positions to be removed from a file.
#[derive(Debug)]
pub struct RemoveEdit {
    range: Range<usize>,
    trim_left: bool,
    trim_right: bool,
}

impl RemoveEdit {
    /// Creates a new edit that also trims all whitespace to the left of the range.
    pub fn new_trim_left(range: Range<usize>) -> RemoveEdit {
        RemoveEdit {
            range,
            trim_left: true,
            trim_right: false,
        }
    }

    /// Creates a new edit that also trims all whitespace to the right of the range.
    pub fn new_trim_right(range: Range<usize>) -> RemoveEdit {
        RemoveEdit {
            range,
            trim_left: false,
            trim_right: true,
        }
    }
}

/// Applies a list of [`RemoveEdit`]s to a file.
pub fn apply_remove_edits<W: Write>(
    file: &mut W,
    src: &str,
    mut edits: Vec<RemoveEdit>,
) -> Result<()> {
    for edit in &mut edits {
        if edit.trim_left {
            edit.range.start = src[..edit.range.start]
                .char_indices()
                .rev()
                .take_while(|(_, c)| c.is_whitespace())
                .last()
                .map_or(edit.range.start, |(i, _)| i);
        }

        if edit.trim_right {
            edit.range.end += src[edit.range.end..]
                .char_indices()
                .take_while(|(_, c)| c.is_whitespace())
                .last()
                .map_or(0, |(i, c)| i + c.len_utf8());
        }
    }

    edits.sort_by_key(|e| Reverse(e.range.start));

    for (index, byte) in src.bytes().enumerate() {
        while edits.pop_if(|e| e.range.end <= index).is_some() {}

        if edits.last().is_none_or(|e| e.range.start > index) {
            file.write_all(&[byte])?;
        }
    }

    Ok(())
}

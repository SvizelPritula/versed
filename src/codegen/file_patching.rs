use std::{
    cmp::Reverse,
    fmt,
    fs::OpenOptions,
    io::{BufWriter, Read, Result, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

pub fn add_extention(path: &Path, extension: &str) -> PathBuf {
    let mut filename = path
        .file_name()
        .expect("path to point to a file")
        .to_owned();
    filename.push(extension);
    path.with_file_name(filename)
}

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
    file.write_all("\n".as_bytes())?;
    file.flush()?;

    Ok(())
}

pub struct AddEdit {
    index: usize,
    content: String,
}

impl AddEdit {
    pub fn new(index: usize, content: String) -> AddEdit {
        AddEdit { index, content }
    }
}

pub fn apply_add_edits<W: Write>(file: &mut W, src: &str, mut edits: Vec<AddEdit>) -> Result<()> {
    edits.sort_by_key(|e| Reverse(e.index));

    for (index, byte) in src.bytes().enumerate() {
        while let Some(edit) = edits.pop_if(|e| e.index == index) {
            file.write_all(edit.content.as_bytes())?;
        }

        file.write(&[byte])?;
    }

    Ok(())
}

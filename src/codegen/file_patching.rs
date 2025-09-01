use std::{
    fmt,
    fs::OpenOptions,
    io::{BufWriter, Read, Result, Seek, SeekFrom, Write},
    path::Path,
};

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
    file.write("\n".as_bytes())?;
    file.flush()?;

    Ok(())
}

use std::{
    fs::{self, File},
    io::{Result, Write},
    path::Path,
};

use crate::{
    ast::TypeSet,
    codegen::file_patching::{add_extention, apply_add_edits},
    migrations::annotate::annotate,
    preprocessing::BasicMetadata,
};

mod annotate;

const OLD_EXTENSION: &str = ".old";

pub fn begin(types: &TypeSet<BasicMetadata>, src: &str, path: &Path) -> Result<()> {
    let edits = annotate(types);

    let old_path = add_extention(path, OLD_EXTENSION);

    let mut file = File::create_new(&old_path)?;
    apply_add_edits(&mut file, src, edits)?;
    file.flush()?;

    fs::copy(&old_path, path)?;

    Ok(())
}

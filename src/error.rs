use std::{
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to access {path}: {error}")]
    Io {
        #[source]
        error: io::Error,
        path: PathBuf,
    },
}

pub trait ResultExt {
    type Result;

    fn with_path<P: AsRef<Path>>(self, path: P) -> Self::Result;
    fn with_stdout(self) -> Self::Result;
    fn with_stderr(self) -> Self::Result;
}

impl<T> ResultExt for Result<T, io::Error> {
    type Result = Result<T, Error>;

    fn with_path<P: AsRef<Path>>(self, path: P) -> Self::Result {
        self.map_err(|error| Error::Io {
            error,
            path: path.as_ref().to_path_buf(),
        })
    }

    fn with_stdout(self) -> Self::Result {
        self.map_err(|error| Error::Io {
            error,
            path: "standard output".into(),
        })
    }

    fn with_stderr(self) -> Self::Result {
        self.map_err(|error| Error::Io {
            error,
            path: "standard error".into(),
        })
    }
}

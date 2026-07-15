//! Contains Versed's [`enum@Error`] type and some helpers.

use std::{
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

/// An error that occurred during the execution of the CLI.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to access {path}: {error}")]
    Io {
        #[source]
        error: io::Error,
        path: PathBuf,
    },
    #[error("The file is malformed")]
    MalformedFile,
}

/// Provides some extension methods on [`Result<T, E>`] where `E` = [`io::Error`].
pub trait ResultExt {
    type Result;

    /// Turns the error into [`Error::Io`] with the specified path.
    fn with_path<P: AsRef<Path>>(self, path: P) -> Self::Result;
    /// Turns the error into [`Error::Io`] with the path set to "standard output".
    fn with_stdout(self) -> Self::Result;
    /// Turns the error into [`Error::Io`] with the path set to "standard error".
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

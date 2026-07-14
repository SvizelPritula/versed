use std::{io::BufWriter, ops::Range, slice};

use anstream::stderr;
use ariadne::Source;

use crate::error::{Error, ResultExt};

type Report<'filename> = ariadne::Report<'static, (&'filename str, Range<usize>)>;

#[derive(Debug, Default)]
pub struct Reports<'filename> {
    reports: Vec<Report<'filename>>,
    has_fatal: bool,
}

impl<'filename> Reports<'filename> {
    pub fn add_fatal(&mut self, report: Report<'filename>) {
        self.reports.push(report);
        self.has_fatal = true;
    }

    pub fn add_nonfatal(&mut self, report: Report<'filename>) {
        self.reports.push(report);
    }

    pub fn extend_fatal<I: IntoIterator<Item = Report<'filename>>>(&mut self, reports: I) {
        self.reports.extend(reports.into_iter().inspect(|_| {
            // A bit of a hack, but I don't see a better way to do this, or why it wouldn't work.
            self.has_fatal = true;
        }));
    }

    pub fn has_fatal(&self) -> bool {
        self.has_fatal
    }

    pub fn has_any(&self) -> bool {
        !self.reports.is_empty()
    }
}

impl<'a, 'filename> IntoIterator for &'a Reports<'filename> {
    type Item = &'a Report<'filename>;
    type IntoIter = slice::Iter<'a, Report<'filename>>;

    fn into_iter(self) -> Self::IntoIter {
        self.reports.iter()
    }
}

pub fn handle_reports(reports: &Reports, filename: &str, src: &str) -> Result<(), Error> {
    if reports.has_any() {
        let mut stream = BufWriter::new(stderr().lock());
        let mut cache = (filename, Source::from(src));

        for report in reports {
            report.write(&mut cache, &mut stream).with_stderr()?;
        }
    }

    if reports.has_fatal() {
        Err(Error::MalformedFile)
    } else {
        Ok(())
    }
}

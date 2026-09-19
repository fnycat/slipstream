use std::{
    backtrace::Backtrace,
    fmt::{self, Display},
    ops::Range,
};

use thiserror::Error;

#[derive(Debug, Error, Default)]
pub struct UnsupportedError {
    pub reason: String,
    pub location: Option<u64>,
}

impl Display for UnsupportedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(location) = self.location {
            write!(
                f,
                "{} is not supported, at location {location}",
                self.reason
            )
        } else {
            write!(f, "{} is not supported", self.reason)
        }
    }
}

#[derive(Debug, Error, Default)]
pub struct CorruptionError {
    pub reason: String,
    pub location: Option<u64>,
}

impl Display for CorruptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = self.location {
            write!(f, "{} at location {}", self.reason, location)
        } else {
            f.write_str(&self.reason)
        }
    }
}

#[derive(Debug, Error, Default)]
pub struct IncorrectFormat {
    pub expected_magic: Vec<u8>,
    pub found_magic: Vec<u8>,
    pub location: Option<u64>,
}

impl Display for IncorrectFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let expected_string = String::from_utf8_lossy(&self.expected_magic);
        let found_string = String::from_utf8_lossy(&self.found_magic);

        if let Some(location) = self.location {
            write!(
                f,
                "expected magic {:?} (`{}`) but found {:?} (`{}`), at location {location}",
                self.expected_magic, expected_string, self.found_magic, found_string
            )
        } else {
            write!(
                f,
                "expected magic {:?} (`{}`) but found {:?} (`{}`)",
                self.expected_magic, expected_string, self.found_magic, found_string
            )
        }
    }
}

#[derive(Debug, Error, Clone, Default)]
pub struct RangeError {
    pub requested: u64,
    pub range: Range<u64>,
    pub location: Option<u64>,
}

impl Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(location) = self.location {
            write!(
                f,
                "{} out of range {}...{}, at location {location}",
                self.requested, self.range.start, self.range.end
            )
        } else {
            write!(
                f,
                "{} out of range {}...{}",
                self.requested, self.range.start, self.range.end
            )
        }
    }
}

#[derive(Debug, Error, Clone, Default)]
pub struct InvalidInputError {
    pub reason: String,
    pub location: Option<u64>,
}

impl Display for InvalidInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(location) = self.location {
            write!(f, "{}, at location {location}", self.reason)
        } else {
            f.write_str(&self.reason)
        }
    }
}

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("unsupported: {source}")]
    Unsupported {
        #[from]
        source: UnsupportedError,
        backtrace: Backtrace,
    },
    #[error("incorrect format: {source}")]
    IncorrectFormat {
        #[from]
        source: IncorrectFormat,
        backtrace: Backtrace,
    },
    #[error("corruption error: {source}")]
    Corrupted {
        #[from]
        source: CorruptionError,
        backtrace: Backtrace,
    },
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[error("invalid utf-8 string: {source}")]
    InvalidString {
        #[from]
        source: std::string::FromUtf8Error,
        backtrace: Backtrace,
    },
    #[error("out of range: {source}")]
    OutOfRange {
        #[from]
        source: RangeError,
        backtrace: Backtrace,
    },
    #[error("eframe error: {source}")]
    EframeError {
        #[from]
        source: eframe::Error,
        backtrace: Backtrace,
    },
    #[error("invalid input: {source}")]
    InvalidInput {
        #[from]
        source: InvalidInputError,
        backtrace: Backtrace,
    },
    #[error("channel send error: {source}")]
    SendError {
        #[from]
        source: futures::channel::mpsc::SendError,
        backtrace: Backtrace,
    },
}

pub type EditorResult<T> = Result<T, EditorError>;

impl<T> From<futures::channel::mpsc::TrySendError<T>> for EditorError {
    fn from(value: futures::channel::mpsc::TrySendError<T>) -> Self {
        value.into_send_error().into()
    }
}

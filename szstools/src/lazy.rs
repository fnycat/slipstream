use std::io::Cursor;

use crate::{arc::FileType, error::EncodingResult};

#[derive(Clone, PartialEq)]
pub enum Lazy<T, R> {
    Deferred {
        raw: R,
        parser: fn(&mut R) -> EncodingResult<T>,
    },
    Parsed(T),
}

impl<T: std::fmt::Debug, R: std::fmt::Debug> std::fmt::Debug for Lazy<T, R> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Deferred { raw, .. } => fmt.debug_set().entry(raw).finish_non_exhaustive(),
            Self::Parsed(x) => fmt.debug_set().entry(x).finish(),
        }
    }
}

impl<P, R> Lazy<P, R> {
    pub fn deferred(raw: R, parser: fn(&mut R) -> EncodingResult<P>) -> Self {
        Self::Deferred { raw, parser }
    }

    pub fn parsed(x: P) -> Self {
        Self::Parsed(x)
    }

    pub fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred { .. })
    }

    pub fn get(&self) -> Option<&P> {
        match self {
            Self::Parsed(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut P> {
        match self {
            Self::Parsed(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_or_parse(&mut self) -> EncodingResult<&mut P> {
        Ok(match self {
            Self::Parsed(x) => x,
            Self::Deferred { raw, parser } => {
                let parsed = parser(raw)?;

                *self = Self::Parsed(parsed);
                let Self::Parsed(parsed) = self else {
                    unreachable!()
                };

                parsed
            }
        })
    }
}

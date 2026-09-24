use crate::error::{EditorError, EditorResult, InvalidInputError};

pub const FORCE_EAGER_EVALUATION: bool = false;

pub struct DeferPayload<P, F, T>
where
    F: FnOnce(P) -> EditorResult<T>,
{
    payload: Option<(P, F)>,
}

impl<P, F, T> DeferParser<T> for DeferPayload<P, F, T>
where
    P: Send + Sync,
    T: Send + Sync,
    F: FnOnce(P) -> EditorResult<T> + Send + Sync,
{
    fn load(&mut self) -> EditorResult<T> {
        let (data, parse_fn) = self.payload.take().ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: String::from("deferred node has no payload"),
                ..Default::default()
            })
        })?;

        tracing::debug!("Lazily parsing node");

        (parse_fn)(data)
    }
}

pub trait DeferParser<T>: Send + Sync {
    fn load(&mut self) -> EditorResult<T>;
}

pub enum Deferred<T> {
    Deferred(Box<dyn DeferParser<T>>),
    Evaluated(T),
}

impl<T> Deferred<T>
where
    T: Send + Sync + 'static,
{
    /// Creates deferred content.
    ///
    /// In other words, the given `data` will be parsed using the `parse_fn` only
    /// when requested.
    pub fn defer<P, F>(data: P, parse_fn: F) -> EditorResult<Self>
    where
        P: Send + Sync + 'static,
        F: FnOnce(P) -> EditorResult<T> + Send + Sync + 'static,
    {
        if FORCE_EAGER_EVALUATION {
            let eval = parse_fn(data)?;
            return Ok(Self::Evaluated(eval));
        }

        Ok(Self::Deferred(Box::new(DeferPayload {
            payload: Some((data, parse_fn)),
        })))
    }

    /// Runs the given closure, and returns its output, if the object has been evaluated.
    ///
    /// Returns `None` otherwise.
    pub fn inspect_ref<F, O>(&self, peek_fn: F) -> Option<O>
    where
        F: FnOnce(&T) -> O,
    {
        match self {
            Self::Evaluated(x) => Some(peek_fn(x)),
            _ => None,
        }
    }

    pub fn inspect_mut<F, O>(&mut self, peek_fn: F) -> Option<O>
    where
        F: FnOnce(&mut T) -> O,
    {
        match self {
            Self::Evaluated(x) => Some(peek_fn(x)),
            _ => None,
        }
    }

    pub fn evaluated(data: T) -> Self {
        Self::Evaluated(data)
    }

    pub fn evaluate(&mut self) -> EditorResult<&mut T> {
        Ok(match self {
            Self::Evaluated(x) => x,
            Self::Deferred(payload) => {
                let eval = payload.load().unwrap();
                *self = Self::Evaluated(eval);

                let Self::Evaluated(eval) = self else {
                    unreachable!()
                };
                eval
            }
        })
    }

    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Evaluated(x) => Some(x),
            _ => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Evaluated(x) => Some(x),
            _ => None,
        }
    }

    pub fn is_deferred(&self) -> bool {
        matches!(self, Self::Deferred(_))
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Deferred<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Evaluated(x) => fmt.debug_set().entry(x).finish(),
            Self::Deferred(_) => fmt.debug_set().finish_non_exhaustive(),
        }
    }
}

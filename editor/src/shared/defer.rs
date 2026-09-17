pub struct DeferPayload<P, F, T>
where
    F: FnOnce(P) -> eyre::Result<T>,
{
    payload: Option<(P, F)>,
}

impl<P, F, T> DeferParser<T> for DeferPayload<P, F, T>
where
    F: FnOnce(P) -> eyre::Result<T>,
{
    fn load(&mut self) -> eyre::Result<T> {
        let (data, parse_fn) = self
            .payload
            .take()
            .ok_or_else(|| eyre::eyre!("deferred node has no payload"))?;

        (parse_fn)(data)
    }
}

pub trait DeferParser<T> {
    fn load(&mut self) -> eyre::Result<T>;
}

pub enum Deferred<T> {
    Deferred(Box<dyn DeferParser<T>>),
    Evaluated(T),
}

impl<T: 'static> Deferred<T> {
    /// Creates deferred content.
    ///
    /// In other words, the given `data` will be parsed using the `parse_fn` only
    /// when requested.
    pub fn defer<P: 'static, F: 'static>(data: P, parse_fn: F) -> Self
    where
        F: FnOnce(P) -> eyre::Result<T>,
    {
        Self::Deferred(Box::new(DeferPayload {
            payload: Some((data, parse_fn)),
        }))
    }

    pub fn evaluated(data: T) -> Self {
        Self::Evaluated(data)
    }

    pub fn evaluate(&mut self) -> eyre::Result<&mut T> {
        Ok(match self {
            Self::Evaluated(x) => x,
            Self::Deferred(payload) => {
                let eval = payload.load()?;
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

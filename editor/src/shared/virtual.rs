use std::{any::Any, cell::RefCell, collections::HashMap, num::NonZeroUsize, rc::Rc};

pub type InspectableRc = Box<dyn Inspectable>;

pub trait LazyParser {
    fn parse(&mut self) -> eyre::Result<InspectableRc>;
}

pub struct LazyPayload<T, F>
where
    F: FnOnce(T) -> eyre::Result<InspectableRc>,
{
    pub inner: Option<(T, F)>,
}

impl<T, F> LazyPayload<T, F>
where
    F: FnOnce(T) -> eyre::Result<InspectableRc>,
{
    pub fn new(payload: T, parser: F) -> Self {
        Self {
            inner: Some((payload, parser)),
        }
    }
}

impl<T, F> LazyParser for LazyPayload<T, F>
where
    F: FnOnce(T) -> eyre::Result<InspectableRc>,
{
    fn parse(&mut self) -> eyre::Result<InspectableRc> {
        let (payload, parser) = self
            .inner
            .take()
            .ok_or_else(|| eyre::eyre!("parser has already been used"))?;

        parser(payload)
    }
}

pub trait Inspectable: std::fmt::Debug {
    fn draw_properties(&mut self, ui: &mut egui::Ui);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CacheId(NonZeroUsize);

impl std::fmt::Display for CacheId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub enum LazyInspectable {
    Deferred(Box<dyn LazyParser>),
    Parsed(InspectableRc),
}

impl LazyInspectable {
    /// Forces this inspectable to be evaluated, allowing it to be rendered in the inspector window.
    ///
    /// This should be called before attempting to use the inspectable.
    pub fn evaluate(&mut self) -> eyre::Result<()> {
        match self {
            Self::Deferred(payload) => {
                let parsed = payload.parse()?;
                *self = Self::Parsed(parsed);
            }
            _ => {}
        }

        Ok(())
    }

    pub fn get_parsed(&self) -> Option<&dyn Inspectable> {
        match self {
            Self::Parsed(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    pub fn get_parsed_mut(&mut self) -> Option<&mut dyn Inspectable> {
        match self {
            Self::Parsed(x) => Some(x.as_mut()),
            _ => None,
        }
    }
}

pub struct CacheStore {
    next_id: usize,
    cache: HashMap<CacheId, LazyInspectable>,
}

impl CacheStore {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            cache: HashMap::new(),
        }
    }

    pub fn insert_deferred<T: 'static, F>(&mut self, lazy_payload: LazyPayload<T, F>) -> CacheId
    where
        F: FnOnce(T) -> eyre::Result<Box<dyn Inspectable>> + 'static,
    {
        let id = self.next_id();
        self.cache
            .insert(id, LazyInspectable::Deferred(Box::new(lazy_payload)));
        id
    }

    pub fn next_id(&mut self) -> CacheId {
        self.next_id += 1;
        CacheId(NonZeroUsize::new(self.next_id - 1).unwrap())
    }

    pub fn get_mut(&mut self, cache_id: CacheId) -> eyre::Result<&mut dyn Inspectable> {
        let lazy = self
            .cache
            .get_mut(&cache_id)
            .ok_or_else(|| eyre::eyre!("did not find cache entry {cache_id} in cache"))?;

        lazy.evaluate()?;
        Ok(lazy.get_parsed_mut().unwrap())
    }

    pub fn get(&mut self, cache_id: CacheId) -> eyre::Result<&dyn Inspectable> {
        let lazy = self
            .cache
            .get_mut(&cache_id)
            .ok_or_else(|| eyre::eyre!("did not find cache entry {cache_id} in cache"))?;

        lazy.get_parsed()
            .ok_or_else(|| eyre::eyre!("inspectable has not been evaluated"))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualNode {
    pub label: String,
    pub kind: VirtualNodeKind,
    pub content: Option<CacheId>,
    pub children: Vec<VirtualNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VirtualNodeKind {
    /// This virtual node can contain other nodes.
    ///
    /// This is used for both directories and files that contain multiple subfiles/sections.
    Container,
    /// This is the final node in this branch.
    ///
    /// This is used for files that are not split up any further.
    Terminal,
}

use std::{any::Any, collections::HashMap, num::NonZeroUsize};

pub trait LazyParser {
    fn parse(self) -> eyre::Result<Box<dyn Inspectable>>;
}

pub struct LazyPayload<T, F>
where
    F: FnOnce(T) -> eyre::Result<Box<dyn Inspectable>>,
{
    pub payload: T,
    pub parser: F,
}

impl<T, F> LazyParser for LazyPayload<T, F>
where
    F: FnOnce(T) -> eyre::Result<Box<dyn Inspectable>>,
{
    fn parse(self) -> eyre::Result<Box<dyn Inspectable>> {
        (self.parser)(self.payload)
    }
}

pub trait Inspectable {}

impl Inspectable for () {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(NonZeroUsize);

pub enum Lazy<T> {
    Deferred(Box<dyn LazyParser>),
    Parsed(T),
}

pub struct ResourceCache {
    next_id: usize,
    cache: HashMap<ResourceId, Lazy<Box<dyn Inspectable>>>,
}

impl ResourceCache {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            cache: HashMap::new(),
        }
    }

    pub fn insert_deferred<T: 'static, F>(&mut self, lazy_payload: LazyPayload<T, F>) -> ResourceId
    where
        F: FnOnce(T) -> eyre::Result<Box<dyn Inspectable>> + 'static,
    {
        let id = self.next_id();
        self.cache
            .insert(id, Lazy::Deferred(Box::new(lazy_payload)));
        id
    }

    pub fn next_id(&mut self) -> ResourceId {
        self.next_id += 1;
        ResourceId(NonZeroUsize::new(self.next_id - 1).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualNode {
    pub label: String,
    pub kind: VirtualNodeKind,
    pub content: Option<ResourceId>,
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

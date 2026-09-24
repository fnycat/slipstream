use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use parking_lot::{ArcRwLockReadGuard, MappedMutexGuard, Mutex, MutexGuard, RawRwLock, RwLock};

use crate::node::node::{Inspectable, InspectableReadGuard, VirtualNode};
use crate::shared::util::AssertSendSync;
use std::fmt;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VirtualNodeId(NonZeroUsize);

impl fmt::Display for VirtualNodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type VirtualNodeMap = Arc<VirtualRefCacheMap>;
pub type VirtualNodeRef = Arc<RwLock<VirtualNode>>;

pub struct ChildrenReadGuard {
    inner: ArcRwLockReadGuard<RawRwLock, VirtualNode>,
}

impl ChildrenReadGuard {
    pub fn into_inner(self) -> ArcRwLockReadGuard<RawRwLock, VirtualNode> {
        self.inner
    }
}

impl From<Arc<RwLock<VirtualNode>>> for ChildrenReadGuard {
    fn from(value: Arc<RwLock<VirtualNode>>) -> Self {
        Self {
            inner: value.read_arc(),
        }
    }
}

impl Deref for ChildrenReadGuard {
    type Target = [VirtualNodeId];

    fn deref(&self) -> &Self::Target {
        self.inner
            .body
            .get()
            .map(|body| body.children.as_slice())
            .expect("virtual node was deferred")
    }
}

impl AsRef<[VirtualNodeId]> for ChildrenReadGuard {
    fn as_ref(&self) -> &[VirtualNodeId] {
        self.deref()
    }
}

/// Maps between node IDs and the nodes that the IDs refer to.
///
/// This allows the editor to quickly access referenced nodes without
/// having to traverse the entire node tree.
#[derive(Default)]
pub struct VirtualRefCacheMap {
    next_id: AtomicUsize,
    /// Nodes are stored in refcells to enable interior mutability.
    /// This ensures that nodes can be inserted into the cache while other nodes are being used.
    refs: DashMap<VirtualNodeId, VirtualNodeRef>,
}

impl VirtualRefCacheMap {
    pub fn new() -> VirtualRefCacheMap {
        Self {
            next_id: AtomicUsize::new(1),
            refs: DashMap::new(),
        }
    }

    pub fn next_id(&self) -> VirtualNodeId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        VirtualNodeId(NonZeroUsize::new(id).unwrap())
    }

    /// Returns the node with the given ID.
    ///
    /// If the node does not exist (or became stale), `None` is returned.
    pub fn get(&self, id: VirtualNodeId) -> Option<VirtualNodeRef> {
        self.refs.get(&id).map(|node| Arc::clone(&node))
    }

    pub fn get_children(&self, id: VirtualNodeId) -> Option<ChildrenReadGuard> {
        let node = self.get(id)?;
        Some(ChildrenReadGuard::from(node))
    }

    /// Loads the inspectable of the given node.
    pub fn get_inspectable<T: Inspectable>(
        &self,
        id: VirtualNodeId,
    ) -> Option<InspectableReadGuard<T>> {
        let node = self.get(id)?;
        Some(InspectableReadGuard::from(node))
    }

    pub fn insert(&self, id: VirtualNodeId, node: VirtualNode) {
        self.refs.insert(id, Arc::new(RwLock::new(node)));
    }
}

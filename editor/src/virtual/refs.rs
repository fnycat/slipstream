use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};

use crate::shared::util::AssertSend;
use crate::r#virtual::node::VirtualNode;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VirtualNodeId(NonZeroUsize);

impl fmt::Display for VirtualNodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type VirtualRefCache = Arc<VirtualRefCacheMap>;
pub type VirtualNodeRef = Arc<Mutex<VirtualNode>>;

/// Maps between node IDs and the nodes that the IDs refer to.
///
/// This allows the editor to quickly access referenced nodes without
/// having to traverse the entire node tree.
pub struct VirtualRefCacheMap {
    next_id: AtomicUsize,
    /// Nodes are stored in refcells to enable interior mutability.
    /// This ensures that nodes can be inserted into the cache while other nodes are being used.
    refs: DashMap<VirtualNodeId, VirtualNodeRef>,
}

impl AssertSend for VirtualRefCacheMap {}

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

    pub fn insert(&self, id: VirtualNodeId, node: VirtualNode) {
        self.refs.insert(id, Arc::new(Mutex::new(node)));
    }
}

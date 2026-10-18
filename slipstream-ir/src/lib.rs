pub mod arc;
pub mod brres;
pub mod compression;
pub mod encoding;
pub mod fixup;
pub mod index;
pub mod kcl;
pub mod kmp;
pub mod layout;
pub mod node;
pub mod tpl;
pub mod util;
pub mod visitor;

/// The maximum amount of capacity a deserializer will preallocate.
///
/// This is used to prevent OOM errors from malformed files.
pub const SAFE_ALLOCATION_CAPACITY: usize = 1_000;

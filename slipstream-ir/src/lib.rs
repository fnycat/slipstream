pub mod arc;
pub mod brres;
pub mod chr;
pub mod clr;
pub mod encoding;
pub mod fixup;
pub mod gx;
pub mod index;
pub mod layout;
pub mod mdl;
pub mod node;
pub mod pat0;
pub mod tex;
pub mod tpl;
pub mod util;
pub mod visitor;
pub mod yaz0;

/// The maximum amount of capacity a deserializer will preallocate.
///
/// This is used to prevent OOM errors from malformed files.
pub const SAFE_ALLOCATION_CAPACITY: usize = 1_000;

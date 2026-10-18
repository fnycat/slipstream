use std::sync::Arc;

use slipstream_shared::cursor::RefCursor;
use slipstream_shared::error::{SlipstreamResult, UnsupportedError};

use crate::arc::ARC_MAGIC;
use crate::brres::{BRRES_MAGIC, CHR_MAGIC, CLR_MAGIC, MDL_MAGIC, TEX_MAGIC};
use crate::compression::{self, YAZ0_MAGIC};
use crate::kmp::KMP_MAGIC;
use crate::node::arena::{IrArena, IrNodeKey};
use crate::tpl::TPL_MAGIC;
use crate::{arc, brres, kmp, tpl};
/// Deserializes a possibly YAZ0-compressed file.
///
/// After decompressing, this forwards the call to [`deserialize_unknown_root`]
pub fn deserialize_maybe_compressed(
    mut reader: RefCursor<[u8]>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    // Is this file compressed?
    if reader.slice(..4)? == YAZ0_MAGIC {
        // then decompress it.
        reader = RefCursor::new(Arc::from(compression::decompress(&mut reader)?));
    }

    deserialize_unknown_root(&mut reader, arena, name)
}

/// Deserializes an uncompressed file.
///
/// If the file may be compressed, call [`deserialize_maybe_compressed`].
///
/// This function works with OS level files, not files within archives.
pub fn deserialize_unknown_root(
    reader: &mut RefCursor<[u8]>,
    arena: &IrArena,
    name: String
) -> SlipstreamResult<IrNodeKey> {
    let magic: &[u8; 4] = reader
        .slice(..4)?
        .try_into()
        .expect("array of size 4 does not have size 4?");

    tracing::trace!("Root file magic is {}", String::from_utf8_lossy(magic));

    let contents = match *magic {
        ARC_MAGIC => arc::deserialize(reader, None, arena, name)?,
        BRRES_MAGIC => brres::deserialize(reader, None, arena, name)?,
        MDL_MAGIC => brres::mdl::deserialize(reader, None, arena, name)?,
        CHR_MAGIC => brres::chr::deserialize(reader, None, arena, name)?,
        CLR_MAGIC => brres::clr::deserialize(reader, None, arena, name)?,
        TEX_MAGIC => brres::tex::deserialize(reader, None, arena, name)?,
        TPL_MAGIC => tpl::deserialize(reader, None, arena, name)?,
        KMP_MAGIC => kmp::deserialize(reader, None, arena, name)?,
        _ => {
            return Err(UnsupportedError {
                reason: format!(
                    "unknown or unsupported file magic: `{}`",
                    String::from_utf8_lossy(magic)
                ),
                location: Some(reader.position())
            }
            .into());
        }
    };

    Ok(contents)
}

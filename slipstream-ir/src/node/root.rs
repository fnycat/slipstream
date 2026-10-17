use std::sync::Arc;

use slipstream_shared::cursor::RefCursor;
use slipstream_shared::error::{SlipstreamResult, UnsupportedError};

use crate::node::arena::{IrArena, IrNodeKey};
use crate::yaz0::{self, YAZ0_MAGIC};
use crate::{arc, brres, chr, mdl, tex, tpl};
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
        reader = RefCursor::new(Arc::from(yaz0::decompress(&mut reader)?));
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
        arc::ARC_MAGIC => arc::deserialize(reader, None, arena, name)?,
        brres::BRRES_MAGIC => brres::deserialize(reader, None, arena, name)?,
        mdl::MDL_MAGIC => mdl::deserialize(reader, None, arena, name)?,
        chr::CHR_MAGIC => chr::deserialize(reader, None, arena, name)?,
        tex::TEX_MAGIC => tex::deserialize(reader, None, arena, name)?,
        tpl::TPL_MAGIC => tpl::deserialize(reader, None, arena, name)?,
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

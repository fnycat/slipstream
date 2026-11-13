use std::collections::HashMap;
use std::io::Write;
use std::marker::PhantomData;

use byteorder::{BigEndian, WriteBytesExt};
use slipstream_shared::SlipstreamResult;
use slipstream_shared::cursor::MutCursor;

/// A very clear placeholder that is easily found in a hex editor.
pub const PLACEHOLDER: u32 = 0xBEEFBABE;

/// Supertrait of `ByteOrder` that adds the ability to check whether the order is big or little endian, as `byteorder` itself does not provide this.
pub trait ByteOrderExt: byteorder::ByteOrder {
    /// Whether this type is big endian.
    const IS_BIG_ENDIAN: bool;
}

impl ByteOrderExt for byteorder::BigEndian {
    const IS_BIG_ENDIAN: bool = true;
}

impl ByteOrderExt for byteorder::LittleEndian {
    const IS_BIG_ENDIAN: bool = false;
}

/// A handle to an empty reserved space in the cursor, that can be written to at a later time.
///
/// `byteorder` does not seem to provide any way to determine the actual endianness from a `ByteOrder` generic
/// so we use a generic bool instead.
pub struct WriteHandle<T, B> {
    location: usize,
    _marker: PhantomData<(T, B)>
}

impl<T: num::traits::ToBytes, B: ByteOrderExt> WriteHandle<T, B> {
    /// Consumes the handle, writing into the empty range.
    pub fn finish(self, cursor: &mut MutCursor, value: T) -> SlipstreamResult<()> {
        let writer = cursor.slice_mut(self.location..self.location + size_of::<T>())?;
        if B::IS_BIG_ENDIAN {
            writer.copy_from_slice(value.to_be_bytes().as_ref());
        } else {
            writer.copy_from_slice(value.to_le_bytes().as_ref());
        }

        Ok(())
    }
}

/// Adds deferred operations to the cursor, which allow reserving space for values that are not known yet.
pub trait DeferredWriteExt {
    /// Reserves space for a i32, returning a handle to write into the space at a later time.
    fn write_i32_def<B: ByteOrderExt>(
        &mut self,
        value: i32
    ) -> SlipstreamResult<WriteHandle<i32, B>>;

    /// Reserves space for a u32, returning a handle to write into the space at a later time.
    fn write_u32_def<B: ByteOrderExt>(
        &mut self,
        value: u32
    ) -> SlipstreamResult<WriteHandle<u32, B>>;
}

impl DeferredWriteExt for MutCursor {
    fn write_i32_def<B: byteorder::ByteOrder>(
        &mut self,
        value: i32
    ) -> SlipstreamResult<WriteHandle<i32, B>> {
        self.write_u32::<B>(PLACEHOLDER)?;

        Ok(WriteHandle {
            location: self.len() - 4,
            _marker: PhantomData
        })
    }

    fn write_u32_def<B: byteorder::ByteOrder>(
        &mut self,
        value: u32
    ) -> SlipstreamResult<WriteHandle<u32, B>> {
        self.write_u32::<B>(PLACEHOLDER)?;

        Ok(WriteHandle {
            location: self.len() - 4,
            _marker: PhantomData
        })
    }
}

/// Keeps track of the current entries in the string pool.
///
/// This is only used for serialisation.
#[derive(Default)]
pub struct StringPool {
    /// Maps a string to an index into the string pool.
    strings: HashMap<String, u32>,
    buffer: MutCursor
}

impl StringPool {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            buffer: MutCursor::new()
        }
    }

    /// Consumes the pool, returning the buffer that was created.
    #[inline]
    pub fn finish(self) -> MutCursor {
        self.buffer
    }

    /// Inserts a string into the pool.
    pub fn insert(&mut self, string: &str) -> SlipstreamResult<u32> {
        // Not using the entry API here to avoid unnecessary cloning of the string.
        Ok(match self.strings.get(string) {
            Some(i) => *i,
            None => {
                // Insert the string
                let index = self.buffer.len() as u32;

                // Add length prefix
                self.buffer.write_u32::<BigEndian>(string.len() as u32)?;
                self.buffer.write_all(string.as_bytes())?;
                self.buffer.write_u8(0)?; // Null terminator

                index
            }
        })
    }
}

use std::io::{self, SeekFrom, Write};
use std::ops::{Bound, RangeBounds};
use std::sync::Arc;

use crate::{SlipstreamResult, verify};

/// Estimates the size of this type when serialized into a buffer.
/// This is used to preallocate.
pub trait SizeEstimate {
    fn estimate_size(&self) -> usize;
}

#[derive(Default)]
pub struct MutCursor {
    inner: Vec<u8>
}

impl MutCursor {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Write for MutCursor {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_all(buf)?;
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        let mut written = 0;
        for buf in bufs {
            self.write_all(buf)?;
            written += buf.len();
        }
        Ok(written)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A `RangedCursor` is very similar to the std's [`Cursor`]
/// but instead stores its contents in a reference counter.
///
/// This allows the cursor to very cheaply be cloned.
///
/// The `*_absolute` methods ignore the current position and operate on the entire underlying buffer.
///
/// [`Cursor`]: std::io::Cursor
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RefCursor<T: ?Sized> {
    inner: Arc<T>,
    /// The current position of the cursor.
    pos: u64
}

impl<T: ?Sized> RefCursor<T> {
    pub fn new(inner: Arc<T>) -> Self {
        Self { inner, pos: 0 }
    }

    pub fn into_inner(self) -> Arc<T> {
        self.inner
    }

    /// The current position of the cursor.
    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the current position of the cursor.
    ///
    /// This position is relative to the start of the cursor slice.
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T: AsRef<[u8]> + ?Sized> RefCursor<T> {
    /// Returns the given range of bytes, taking into account the cursor position.
    ///
    /// This function returns an error if the range is out of bounds of the array.
    pub fn slice<B: RangeBounds<usize>>(&self, range: B) -> SlipstreamResult<&[u8]> {
        let start = range.start_bound();
        let end = range.end_bound();

        let rem = self.remaining();
        let start = match start {
            Bound::Unbounded => 0,
            Bound::Included(&x) => {
                verify!(x < rem, "start bound is larger than slice length ({rem} vs. {x})");
                x
            },
            Bound::Excluded(&x) => {
                verify!(x + 1 < rem, "start bound is larger than slice length ({rem} vs. {x})");
                x + 1
            }
        };

        let end = match end {
            Bound::Unbounded => rem,
            Bound::Included(&x) => {
                verify!(x + 1 < rem, "end bound exceeds slice length ({rem} vs. {x})");
                x + 1
            },
            Bound::Excluded(&x) => {
                verify!(x < rem, "end bound exceeds slice length ({rem} vs. {x}");
                x
            }
        };

        // Then ensure the start index does not exceed the end.
        verify!(start <= end, "start bound is larger than end bound ({start} vs. {end})");

        Ok(&self.remaining_slice()[start..end])
    }

    /// Returns the given range of bytes, ignoring the cursor position.
    ///
    /// This functions clamps the bounds to the size of array, it *does not panic*.
    pub fn slice_absolute<B: RangeBounds<usize>>(&self, range: B) -> &[u8] {
        let start = range.start_bound();
        let end = range.end_bound();

        let rem = self.remaining();
        let start = match start {
            Bound::Unbounded => 0,
            Bound::Included(&x) => x.min(rem - 1),
            Bound::Excluded(&x) => (x + 1).min(rem - 1)
        };

        let end = match end {
            Bound::Unbounded => rem - 1,
            Bound::Included(&x) => x.min(rem - 1),
            Bound::Excluded(&x) => (x - 1).min(rem - 1)
        };

        &self.inner.as_ref().as_ref()[start..=end]
    }

    /// Returns a reference to the bytes remaining in this buffer.
    ///
    /// I.e this buffer will be the bytes in the range `pos...range.end`.
    ///
    /// If the cursor is past the end of the buffer, the remaining buffer will be empty.
    pub fn remaining_slice(&self) -> &[u8] {
        // If the position is past the end of the buffer, return an empty slice
        if self.remaining() > 0 {
            &self.inner.as_ref().as_ref()[self.pos as usize..]
        } else {
            &[]
        }
    }

    /// Returns the length of the underlying buffer, ignoring the cursor position.
    pub fn len(&self) -> usize {
        self.inner.as_ref().as_ref().len()
    }

    /// The length of the remaining buffer.
    #[inline]
    pub fn remaining(&self) -> usize {
        (self.len() as u64).saturating_sub(self.pos) as usize
    }

    /// Dumps all contents to the given file in binary format.
    pub fn dump<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        std::fs::write(path.as_ref(), self.inner.as_ref())
    }
}

impl<T: ?Sized> Clone for RefCursor<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            pos: self.pos
        }
    }
}

/// This is a nearly exact copy of the standard library.
impl<T> io::Read for RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rem = self.remaining_slice();
        let n = std::cmp::min(buf.len(), rem.len());

        buf[..n].copy_from_slice(&rem[..n]);
        self.set_position(self.position() + n as u64);

        Ok(n)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread = 0;
        for buf in bufs {
            let n = self.read(buf)?;
            nread += n;
            if n < buf.len() {
                break;
            }
        }

        Ok(nread)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let rem = self.remaining_slice();
        let n = buf.len();

        if rem.len() < n {
            // Set cursor to EOF
            self.set_position(self.inner.as_ref().as_ref().len() as u64);
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }

        buf.copy_from_slice(&rem[..n]);
        self.set_position(self.position() + buf.len() as u64);

        Ok(())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let rem = self.remaining_slice();
        let n = rem.len();

        buf.reserve(n);
        buf.extend_from_slice(rem);

        self.set_position(self.position() + n as u64);

        Ok(n)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let rem = self.remaining_slice();
        let n = rem.len();

        let content = str::from_utf8(rem)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8"))?;

        buf.reserve(n);
        buf.push_str(content);

        self.set_position(self.position() + n as u64);

        Ok(n)
    }
}

/// This is a nearly exact copy of the standard library.
impl<T> io::Seek for RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized
{
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.set_position(n);
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.as_ref().as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.position(), n)
        };

        match base_pos.checked_add_signed(offset) {
            Some(n) => {
                self.set_position(n);
                Ok(n)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position"
            ))
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.position())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::RefCursor;

    const TEST_BUFFER: [u8; 3] = [1, 2, 3];
    const TEST_SIZE: usize = TEST_BUFFER.len();

    #[test]
    fn slice_inclusive() {
        let cursor = RefCursor::new(Arc::from(&TEST_BUFFER));
        for start in 0..TEST_SIZE {
            for end in 0..TEST_SIZE - 1 {
                let start = start.clamp(0, end);

                let sliced = cursor.slice(start..=end).expect("failed to slice cursor");
                assert_eq!(sliced, &TEST_BUFFER[start..=end]);
            }
        }
    }

    #[test]
    fn slice_exclusive() {
        let cursor = RefCursor::new(Arc::from(&TEST_BUFFER));
        for start in 0..TEST_SIZE {
            for end in 0..TEST_SIZE {
                let start = start.clamp(0, end);

                let sliced = cursor.slice(start..end).expect("failed to slice cursor");
                assert_eq!(sliced, &TEST_BUFFER[start..end]);
            }
        }
    }

    #[test]
    fn slice_end_exclusive() {
        let cursor = RefCursor::new(Arc::from(&TEST_BUFFER));
        for end in 0..TEST_SIZE {
            let sliced = cursor.slice(..end).expect("failed to slice cursor");
            assert_eq!(sliced, &TEST_BUFFER[..end]);
        }
    }

    #[test]
    fn slice_end_inclusive() {
        let cursor = RefCursor::new(Arc::from(&TEST_BUFFER));
        for end in 0..TEST_SIZE - 1 {
            let sliced = cursor.slice(..=end).expect("failed to slice cursor");
            assert_eq!(sliced, &TEST_BUFFER[..=end]);
        }
    }

    #[test]
    fn slice_start_exclusive() {
        let cursor = RefCursor::new(Arc::from(&TEST_BUFFER));
        for start in 0..TEST_SIZE {
            let sliced = cursor.slice(start..).expect("failed to slice cursor");
            assert_eq!(sliced, &TEST_BUFFER[start..]);
        }
    }
}
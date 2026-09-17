use std::{
    io::{self, SeekFrom},
    ops::Range,
    rc::Rc,
};
use std::ops::{Bound, RangeBounds};

/// A `RangedCursor` is very similar to the std's [`Cursor`]
/// but instead stores its contents in a reference counter.
///
/// This allows the cursor to very cheaply be cloned.
///
/// [`Cursor`]: std::io::Cursor
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized,
{
    inner: Rc<T>,
    /// The current position of the cursor.
    pos: u64,
    lower_bound: u64
}

/// This is a nearly exact copy of the standard library.
impl<T> RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized,
{
    pub fn new(inner: Rc<T>) -> Self {
        Self {
            inner,
            pos: 0,
            lower_bound: 0
        }
    }

    /// Returns a cursor that only reads the remaining bytes.
    pub fn tail(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            pos: 0,
            lower_bound: self.position()
        }
    }

    pub fn set_tail(&mut self) {
        self.lower_bound = self.position();
        self.pos = 0;
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the current position of the cursor.
    ///
    /// This position is relative to the start of the cursor slice.
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }

    /// Returns a reference to the bytes remaining in this buffer.
    ///
    /// I.e this buffer will be the bytes in the range `pos...range.end`.
    ///
    /// If the cursor is past the end of the buffer, the remaining buffer will be empty.
    pub fn as_remaining(&self) -> &[u8] {
        &self.inner.as_ref().as_ref()[(self.pos + self.lower_bound) as usize..]
    }

    /// Returns the length of the entire underlying buffer.
    ///
    /// This function completely disregards the slicing mechanics.
    pub fn full_len(&self) -> usize {
        self.inner.as_ref().as_ref().len()
    }

    /// The length of the remaining buffer.
    pub fn remaining_len(&self) -> usize {
        self.as_remaining().len()
    }

    pub fn dump<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        std::fs::write(path.as_ref(), self.inner.as_ref())
    }
}

impl<T> Clone for RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            pos: self.pos,
            lower_bound: self.lower_bound
        }
    }
}

/// This is a nearly exact copy of the standard library.
impl<T> io::Read for RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rem = self.as_remaining();
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
        let rem = self.as_remaining();
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
        let rem = self.as_remaining();
        let n = rem.len();

        buf.reserve(n);
        buf.extend_from_slice(rem);

        self.set_position(self.position() + n as u64);

        Ok(n)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let rem = self.as_remaining();
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
    T: AsRef<[u8]> + ?Sized,
{
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.set_position(n);
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.as_ref().as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.position(), n),
        };

        match base_pos.checked_add_signed(offset) {
            Some(n) => {
                self.set_position(n);
                Ok(n)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.position())
    }
}

pub fn draw_vec_drag_values<T: egui::emath::Numeric, const N: usize>(
    mut input_field_size: egui::Vec2,
    labels: [&str; N],
    values: &mut [T; N],
    ui: &mut egui::Ui,
) {
    input_field_size.x /= N as f32;

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        for i in (0..N).rev() {
            ui.add_sized(input_field_size, egui::DragValue::new(&mut values[i]));
            if labels[i].is_empty() {
                let label_width = ui
                    .painter()
                    .layout_no_wrap(
                        "X:".to_owned(),
                        egui::FontId::default(),
                        egui::Color32::TRANSPARENT,
                    )
                    .rect
                    .width();
                ui.allocate_space(egui::vec2(label_width, input_field_size.y));
            } else {
                ui.label(labels[i]);
            }
        }
    });
}

pub fn draw_inspector_section_header(name: String, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let total_width = ui.available_width();
        let text_width = ui
            .painter()
            .layout_no_wrap(
                name.clone(),
                egui::FontId::default(),
                ui.visuals().text_color(),
            )
            .rect
            .width();

        let padding = 16.0;
        let line_width = ((total_width - text_width - padding) / 2.0).max(0.0);
        let separator_size = egui::vec2(line_width, ui.available_height());

        ui.add_space(0.01 * ui.available_height());
        ui.add_sized(separator_size, egui::Separator::default().horizontal());
        ui.label(name);
        ui.add_sized(separator_size, egui::Separator::default().horizontal());
        ui.add_space(0.01 * ui.available_height());
    });
}

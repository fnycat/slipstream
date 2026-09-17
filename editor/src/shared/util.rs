use std::{
    io::{self, SeekFrom},
    ops::Range,
    rc::Rc,
};

/// A `RangedCursor` is very similar to the std's [`Cursor`]
/// but instead stores its contents in a reference counter.
///
/// This allows the cursor to very cheaply be cloned. The cursor itself
/// keeps track of the bounds of its buffer, allowing cursors to use different
/// sections of the same underlying buffer.
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
    /// The bounds that this cursor should read within.
    range: Range<u64>,
}

/// This is a nearly exact copy of the standard library.
impl<T> RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized,
{
    pub fn new(inner: Rc<T>) -> Self {
        let len = inner.as_ref().as_ref().len() as u64;

        Self {
            inner,
            pos: 0,
            range: 0..len,
        }
    }

    pub fn new_sliced(inner: Rc<T>, range: Range<u64>) -> io::Result<Self> {
        let len = inner.as_ref().as_ref().len() as u64;
        if range.end > len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "slice range is outside of buffer",
            ));
        }

        Ok(Self {
            inner,
            pos: range.start,
            range,
        })
    }

    /// Returns a subset of `self`.
    ///
    /// # Errors
    ///
    /// The function returns an [`InvalidInput`] error when the range is outside of the buffer bounds.
    ///
    /// [`InvalidInput`]: std::io::ErrorKind::InvalidInput
    pub fn slice(&self, range: Range<u64>) -> io::Result<Self> {
        Self::new_sliced(self.inner.clone(), range)
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn split(&self) -> (&[u8], &[u8]) {
        let slice = self.get_ref().as_ref();
        let pos = self.pos.min(slice.len() as u64);
        slice.split_at(pos as usize)
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
            range: self.range.clone(),
        }
    }
}

/// This is a nearly exact copy of the standard library.
impl<T> io::Read for RefCursor<T>
where
    T: AsRef<[u8]> + ?Sized,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = io::Read::read(&mut Self::split(self).1, buf)?;
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
        let result = io::Read::read_exact(&mut Self::split(self).1, buf);

        match result {
            Ok(_) => self.set_position(self.position() + buf.len() as u64),
            Err(_) => self.set_position(self.get_ref().as_ref().len() as u64),
        }

        result
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let content = Self::split(self).1;
        let len = content.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(content);

        self.set_position(self.position() + len as u64);
        Ok(len)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let content = str::from_utf8(Self::split(self).1)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8 encountered"))?;

        let len = content.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        self.set_position(self.position() + len as u64);

        Ok(len)
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
            SeekFrom::End(n) => (self.get_ref().as_ref().len() as u64, n),
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

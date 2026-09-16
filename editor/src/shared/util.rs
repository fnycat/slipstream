pub struct SlidingWindows<'a, T> {
    slice: &'a [T],
    index: usize,
    window_size: usize,
}

impl<'a, T> SlidingWindows<'a, T> {
    pub fn new(slice: &'a [T], window_size: usize) -> Self {
        Self {
            slice,
            window_size,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for SlidingWindows<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        let upper_bound = (self.index + self.window_size).clamp(0, self.slice.len() - 1);
        if self.index >= upper_bound {
            return None;
        }

        let item = &self.slice[self.index..upper_bound];
        self.index += 1;
        Some(item)
    }
}

impl<'a, T> ExactSizeIterator for SlidingWindows<'a, T> {
    fn len(&self) -> usize {
        (self.slice.len() - self.index).div_ceil(self.window_size)
    }
}

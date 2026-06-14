pub struct Walker<'l, T> {
    items: &'l [T],
    len: usize,
    pos: usize,
}

impl<'l, T> Walker<'l, T> {
    pub fn new(items: &'l [T]) -> Self {
        Self {
            items,
            len: items.len(),
            pos: 0,
        }
    }
    pub fn current(&self) -> Option<&T> {
        if self.pos >= self.len {
            return None;
        }
        self.items.get(self.pos)
    }
    pub fn next(&mut self) -> Option<&T> {
        if self.pos < self.len {
            self.pos += 1;
        }
        self.current()
    }
    pub fn drop_tail(&mut self) -> Self {
        let tail = Self {
            items: &self.items[..self.pos],
            len: self.pos,
            pos: 0,
        };
        self.items = &self.items[self.pos..];
        self.pos = 0;
        tail
    }
    pub fn reset(&mut self) {
        self.pos = 0;
    }
}

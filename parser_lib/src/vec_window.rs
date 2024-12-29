use std::fmt::Debug;

pub struct VecWindow<'l, T> {
    vec: &'l Vec<T>,
    start_index: usize,
    end_index: usize,
}

impl<T> VecWindow<'_, T> {
    pub fn is_empty(&self) -> bool {
        self.start_index > self.end_index
    }
    pub fn first(&self) -> Option<&T> {
        if self.start_index > self.end_index {
            None
        } else {
            self.vec.get(self.start_index)
        }
    }
    pub fn last(&self) -> Option<&T> {
        if self.start_index > self.end_index {
            None
        } else {
            self.vec.get(self.end_index)
        }
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        if index > self.end_index - self.start_index {
            None
        } else {
            self.vec.get(index + self.start_index)
        }
    }
    pub fn pop_front(&mut self) -> Option<&T> {
        if self.start_index <= self.end_index {
            let res = self.vec.get(self.start_index);
            self.start_index += 1;
            res
        } else {
            None
        }
    }
    pub fn pop_back(&mut self) -> Option<&T> {
        if self.start_index <= self.end_index {
            let res = self.vec.get(self.end_index);
            self.end_index -= 1;
            res
        } else {
            None
        }
    }
}

impl<'l, T> From<&'l Vec<T>> for VecWindow<'l, T> {
    fn from(vec: &'l Vec<T>) -> Self {
        VecWindow {
            vec,
            start_index: 0,
            end_index: vec.len() - 1,
        }
    }
}

impl<T> Clone for VecWindow<'_, T> {
    fn clone(&self) -> Self {
        VecWindow {
            vec: self.vec,
            start_index: self.start_index,
            end_index: self.end_index,
        }
    }
}

impl<T: Debug> Debug for VecWindow<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.vec[self.start_index..=self.end_index].iter())
            .finish()
    }
}

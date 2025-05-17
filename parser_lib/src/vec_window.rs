use std::cmp::min;
use std::fmt::Debug;

#[derive(Eq, PartialEq)]
pub struct VecWindow<'l, T> {
    vec: &'l Vec<T>,
    /// inclusive
    start_index: usize,
    /// exclusive
    end_index: usize,
}

impl<'l, T> VecWindow<'l, T> {
    pub fn new(vec: &'l Vec<T>, start_index: usize, end_index: usize) -> Option<Self> {
        if start_index > end_index {
            return None;
        }
        if end_index > vec.len() {
            return None;
        }
        Some(VecWindow {
            vec,
            start_index,
            end_index,
        })
    }
    pub fn is_empty(&self) -> bool {
        self.start_index >= self.end_index
    }
    /// Size of the window.
    pub fn size(&self) -> usize {
        self.end_index - self.start_index
    }
    /// Start index, inclusive.
    pub fn start(&self) -> usize {
        self.start_index
    }
    /// End index, exclusive.
    pub fn end(&self) -> usize {
        self.end_index
    }
    /// First element if the window is not empty.
    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.vec[self.start_index])
        }
    }
    /// Last element if the window is not empty.
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.vec[self.end_index - 1])
        }
    }
    /// Get the element at the given index, relative to the start of the window.
    /// If the index is out of bounds, return None.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.size() {
            None
        } else {
            Some(&self.vec[index + self.start_index])
        }
    }
    /// Remove and return the first element of the window.
    pub fn pop_first(&mut self) -> Option<&'l T> {
        if self.is_empty() {
            None
        } else {
            let res = Some(&self.vec[self.start_index]);
            self.start_index += 1;
            res
        }
    }
    /// Remove and return the last element of the window.
    pub fn pop_last(&mut self) -> Option<&'l T> {
        if self.is_empty() {
            None
        } else {
            let res = Some(&self.vec[self.end_index - 1]);
            self.end_index -= 1;
            res
        }
    }
    /// Skip the first n elements of the window.
    /// If n is greater than the size of the window, return an empty window.
    pub fn skip(self, n: usize) -> Self {
        VecWindow {
            vec: self.vec,
            start_index: min(self.start_index + n, self.end_index),
            end_index: self.end_index,
        }
    }
    /// Take the first n elements of the window.
    /// If n is greater than the size of the window, return the whole window.
    pub fn take(self, n: usize) -> Self {
        VecWindow {
            vec: self.vec,
            start_index: self.start_index,
            end_index: min(self.start_index + n, self.end_index),
        }
    }
    /// Increase the start index of the window
    /// only if the new index is inside the window.
    pub fn shrink_start_to(&mut self, new_start: usize) {
        // if new_start == start_index nothing to do
        if new_start > self.start_index && new_start <= self.end_index {
            self.start_index = new_start;
        }
    }
    /// Decrease the end index of the window
    /// only if the new index is inside the window.
    pub fn shrink_end_to(&mut self, new_end: usize) {
        // if new_end == end_index nothing to do
        if new_end < self.end_index && new_end >= self.start_index {
            self.end_index = new_end;
        }
    }
    /// Get the index of the first element that matches the given function.
    pub fn find<F: Fn(&T) -> bool>(&self, f: F) -> Option<usize> {
        for i in self.start_index..self.end_index {
            if f(&self.vec[i]) {
                return Some(i - self.start_index);
            }
        }
        None
    }
    /// Empty the window.
    pub fn empty(self) -> Self {
        VecWindow {
            vec: self.vec,
            start_index: 0,
            end_index: 0,
        }
    }
    /// Snip the window at the given index,
    /// creating two new windows the second one starting at the snip index.
    pub fn snip(self, at: usize) -> Option<(Self, Self)> {
        if at >= self.size() {
            return None;
        }
        Some((
            VecWindow {
                vec: self.vec,
                start_index: self.start_index,
                end_index: self.start_index + at,
            },
            VecWindow {
                vec: self.vec,
                start_index: self.start_index + at,
                end_index: self.end_index,
            },
        ))
    }
    /// Split the window on the given function,
    /// removing the matching elements.
    /// This function wraps [Self::split_including_start].
    pub fn split<F: Fn(&T) -> bool>(self, on: F) -> Vec<Self> {
        self.split_including_start(on)
            .into_iter()
            .enumerate()
            .map(|(i, e)| if i == 0 { e } else { e.skip(1) })
            .collect()
    }
    /// Split the window on the given function,
    /// including the matching elements
    /// in the window after the split.
    pub fn split_including_start<F: Fn(&T) -> bool>(self, on: F) -> Vec<Self> {
        if self.is_empty() {
            return vec![];
        }
        // if the window has only one element,
        // splitting it would just return the same window
        if self.size() <= 1 {
            return vec![self];
        }
        let mut res = Vec::new();
        let mut start = self.start_index;
        // the earlier check makes sure start_index + 1 is valid
        // skip the first element because it's always in the first window
        for i in (self.start_index + 1)..self.end_index {
            if on(&self.vec[i]) {
                res.push(VecWindow {
                    vec: self.vec,
                    start_index: start,
                    end_index: i,
                });
                start = i;
            }
        }
        // push the remaining window
        res.push(VecWindow {
            vec: self.vec,
            start_index: start,
            end_index: self.end_index,
        });
        res
    }
    /// Split the window once on the given function,
    /// removing the matching element.
    pub fn split_once<F: Fn(&T) -> bool>(self, on: F) -> Option<(Self, Self)> {
        for i in self.start_index..self.end_index {
            if on(&self.vec[i]) {
                return Some((
                    VecWindow {
                        vec: self.vec,
                        start_index: self.start_index,
                        end_index: i,
                    },
                    VecWindow {
                        vec: self.vec,
                        start_index: i + 1,
                        end_index: self.end_index,
                    },
                ));
            }
        }
        None
    }
}

impl<'l, T> From<&'l Vec<T>> for VecWindow<'l, T> {
    fn from(vec: &'l Vec<T>) -> Self {
        if vec.is_empty() {
            return VecWindow {
                vec,
                start_index: 0,
                end_index: 0,
            };
        }
        VecWindow {
            vec,
            start_index: 0,
            end_index: vec.len(),
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
            .entries(self.vec[self.start_index..self.end_index].iter())
            .finish()
    }
}

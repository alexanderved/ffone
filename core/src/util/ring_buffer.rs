use std::{iter, slice, vec};

pub struct RingBuffer<T, const N: usize> {
    storage: Vec<T>,
    pos: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            storage: Vec::with_capacity(N),
            pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn read(&self, index: usize) -> Option<&T> {
        self.storage.get(index)
    }

    pub fn read_mut(&mut self, index: usize) -> Option<&mut T> {
        self.storage.get_mut(index)
    }

    pub fn write(&mut self, elem: T) {
        if self.storage.len() < N {
            self.storage.push(elem);
        } else {
            unsafe {
                debug_assert!(self.pos < N);

                *self.storage.get_unchecked_mut(self.pos) = elem;
            }
        }

        self.pos += 1;
        self.pos %= N;
    }

    pub fn clear(&mut self) {
        self.storage.clear();
        self.pos = 0;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }
}

impl<T, const N: usize> IntoIterator for RingBuffer<T, N> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.storage.into_iter())
    }
}

impl<'rb, T, const N: usize> IntoIterator for &'rb RingBuffer<T, N> {
    type IntoIter = Iter<'rb, T>;
    type Item = &'rb T;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.storage.iter())
    }
}

impl<'rb, T, const N: usize> IntoIterator for &'rb mut RingBuffer<T, N> {
    type IntoIter = IterMut<'rb, T>;
    type Item = &'rb mut T;

    fn into_iter(self) -> Self::IntoIter {
        IterMut(self.storage.iter_mut())
    }
}

#[derive(Debug, Clone)]
pub struct IntoIter<T>(vec::IntoIter<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> iter::ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'rb, T>(slice::Iter<'rb, T>);

impl<'rb, T> Iterator for Iter<'rb, T> {
    type Item = &'rb T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> iter::ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug)]
pub struct IterMut<'rb, T>(slice::IterMut<'rb, T>);

impl<'rb, T> Iterator for IterMut<'rb, T> {
    type Item = &'rb mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> iter::ExactSizeIterator for IterMut<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

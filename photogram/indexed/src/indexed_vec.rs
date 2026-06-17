use std::marker::PhantomData;

use super::{Idx, IndexedSlice};

/// An [IndexedVec] is a Vec of items with an index
#[derive(Clone)]
pub struct IndexedVec<I, T, const M: bool>
where
    I: Idx,
{
    array: Vec<T>,
    _phantom: PhantomData<fn(&I)>,
}

impl<I, T, const M: bool> std::fmt::Debug for IndexedVec<I, T, M>
where
    I: Idx,
    T: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.array.fmt(fmt)
    }
}

impl<I, T, const M: bool> std::default::Default for IndexedVec<I, T, M>
where
    I: Idx,
{
    fn default() -> Self {
        let array = vec![];
        Self {
            array,
            _phantom: PhantomData,
        }
    }
}

impl<I, T, const M: bool> std::ops::Index<I> for IndexedVec<I, T, M>
where
    I: Idx,
{
    type Output = T;
    #[track_caller]
    fn index(&self, idx: I) -> &T {
        &self.array[idx.opt_index().unwrap()]
    }
}

impl<I, T, const M: bool> IndexedVec<I, T, M>
where
    I: Idx,
{
    /// Gives the next index that will be assigned when `push` is
    /// called.
    #[inline]
    pub fn next_index(&self) -> I {
        I::from_usize(self.array.len())
    }

    /// Return an [IndexSlice] for the contents
    #[inline(always)]
    pub fn as_slice(&self) -> &IndexedSlice<I, [T], M> {
        IndexedSlice::new(&self.array)
    }

    /// Push a new item onto the vector, and return it's index.
    #[inline]
    pub fn push(&mut self, d: T) -> I {
        let index = self.next_index();
        self.array.push(d);
        index
    }

    /// Get a ref to the item at the provided index, or None for out of bounds.
    #[inline]
    pub fn get(&self, index: I) -> Option<&T> {
        self.as_slice().get(index)
    }
}

impl<'a, I, T, const M: bool> std::iter::IntoIterator for &'a IndexedVec<I, T, M>
where
    I: Idx,
{
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    // Required method
    fn into_iter(self) -> std::slice::Iter<'a, T> {
        self.array.iter()
    }
}

//ip AsRef<[T]> for IndexedVec
impl<I, T, const M: bool> AsRef<[T]> for IndexedVec<I, T, M>
where
    I: Idx,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.array
    }
}

//ip AsRef<IndexedSlice<I, [T]>> for IndexedVec
impl<I, T, const M: bool> AsRef<IndexedSlice<I, [T], M>> for IndexedVec<I, T, M>
where
    I: Idx,
{
    #[inline]
    fn as_ref(&self) -> &IndexedSlice<I, [T], M> {
        IndexedSlice::new(&self.array)
    }
}

//ip Deref for IndexedVec
impl<I, T, const M: bool> std::ops::Deref for IndexedVec<I, T, M>
where
    I: Idx,
{
    type Target = IndexedSlice<I, [T], M>;
    #[inline]
    fn deref(&self) -> &IndexedSlice<I, [T], M> {
        IndexedSlice::new(&self.array)
    }
}

/// Mutable methods for IndexedVec<..,..,true>
impl<I, T> IndexedVec<I, T, true>
where
    I: Idx,
{
    /// Return an [IndexSlice] for the contents
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut IndexedSlice<I, [T], true> {
        IndexedSlice::new_mut(&mut self.array)
    }

    /// Get a ref to the item at the provided index, or None for out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        self.as_mut_slice().get_mut(index)
    }
    // Could add... pop, insert,
}

impl<I, T> std::ops::IndexMut<I> for IndexedVec<I, T, true>
where
    I: Idx,
{
    #[track_caller]
    fn index_mut(&mut self, idx: I) -> &mut T {
        &mut self.array[idx.opt_index().unwrap()]
    }
}

impl<I, T> AsMut<IndexedSlice<I, [T], true>> for IndexedVec<I, T, true>
where
    I: Idx,
{
    #[inline]
    fn as_mut(&mut self) -> &mut IndexedSlice<I, [T], true> {
        IndexedSlice::new_mut(&mut self.array)
    }
}

impl<I, T> std::ops::DerefMut for IndexedVec<I, T, true>
where
    I: Idx,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut IndexedSlice<I, [T], true> {
        IndexedSlice::new_mut(&mut self.array)
    }
}

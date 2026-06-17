use std::marker::PhantomData;

use super::Idx;

/// This is a type-wrapper for a standard slice, which is indexed by a
/// type which support Idx. It should exist only the form
/// `IndexedSlice<I,[D]>` for some data type D
///
/// An `IndexedSlice<[T]>` is created *from* a standard slice; the
/// 'slice' in the IndexedSlice is guaranteed to be bit-copy identical
/// to the underlying slice, as it *is* the underlying slice. Because
/// of this, an `&IndexedSlice<[T]>` is bit-wise identical to an
/// `&[T]`.
///
/// The lifetime of an &IndexedSlice is guaranteed to be less than
/// that of the slice on which it is based, hence this is a safe type.
///
/// This is based heavily on the index_vec::IndexSlice
///
/// The type T is a slice [D]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct IndexedSlice<I, T, const M: bool>
where
    I: Idx,
    T: ?Sized,
{
    _marker: PhantomData<fn(&I)>,

    /// The underlying slice, which should be `[D]`
    ///
    /// Must be last in the type as IndexedSlice is a DST (dynamically
    /// sized type), driven by the slice T
    slice: T,
}

//ip Debug for IndexedSlice<I, T, M>
impl<I, T, const M: bool> std::fmt::Debug for IndexedSlice<I, T, M>
where
    I: Idx,
    T: std::fmt::Debug + ?Sized,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.slice.fmt(fmt)
    }
}

//ip IndexedSlice (mutable)
impl<I, T> IndexedSlice<I, [T], true>
where
    I: Idx,
    T: Sized,
{
    //cp from_slice_mut
    /// Construct a new IndexedSlice by type-wrapping an existing
    /// slice.
    #[inline(always)]
    pub fn from_slice_mut(slice: &mut [T]) -> &mut Self {
        unsafe { &mut *(slice as *mut [T] as *mut Self) }
    }

    //cp new_mut
    /// Construct a new IndexedSlice by type-wrapping an existing
    /// slice, returning a reference that is effectively borrowed from
    /// the argument
    #[inline(always)]
    pub fn new_mut<S: AsMut<[T]>>(slice: &mut S) -> &mut Self {
        Self::from_slice_mut(slice.as_mut())
    }
    //ap get_mut
    /// Get a mutable reference to the item at the provided index, or
    /// None for out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        self.slice.get_mut(index.index())
    }
}

//ip IndexedSlice
impl<I, T, const M: bool> IndexedSlice<I, [T], M>
where
    I: Idx,
    T: Sized,
{
    //cp new
    /// Construct a new IndexedSlice by type-wrapping an existing
    /// slice, returning a reference that is effectively borrowed from
    /// the argument
    #[inline(always)]
    pub fn new<S: AsRef<[T]>>(slice: &S) -> &Self {
        Self::from_slice(slice.as_ref())
    }

    //cp from_slice
    /// Construct a new IndexedSlice by type-wrapping an existing
    /// slice.
    #[inline(always)]
    pub const fn from_slice(slice: &[T]) -> &Self {
        unsafe { &*(slice as *const [T] as *const Self) }
    }

    //ap inner
    /// Returns the slice that this type-wraps
    #[inline(always)]
    pub const fn inner(&self) -> &[T] {
        &self.slice
    }

    //ap len
    /// Returns the length of the wrapped slice
    #[inline]
    pub const fn len(&self) -> usize {
        self.slice.len()
    }

    //ap len
    /// Returns the length of the wrapped slice as an `I`.
    #[inline]
    pub fn len_idx(&self) -> I {
        I::from_usize(self.slice.len())
    }

    //ap is_empty
    /// Returns true if we're empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    //ap iter
    /// Get a iterator over references to our values.
    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
        self.slice.iter()
    }

    //ap enumerate
    /// Get an interator over references with an item of `(I, &T)`
    #[inline(always)]
    pub fn iter_enumerated(&self) -> impl ExactSizeIterator<Item = (I, &T)> {
        self.slice
            .iter()
            .enumerate()
            .map(|(i, t)| (I::from_usize(i), t))
    }

    //ap indices
    /// Get an interator over all our indices.
    #[inline(always)]
    pub fn indices(&self) -> impl ExactSizeIterator<Item = I> {
        (0..self.slice.len()).map(I::from_usize)
    }

    //ap contains
    /// Forwards to the slice's `contains` implementation.
    #[inline]
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.slice.contains(x)
    }

    //ap position
    /// Searches for an element in an iterator, returning its index. This is
    /// equivalent to `Iterator::position`, but returns `I` and not `usize`.
    #[inline(always)]
    pub fn position<F: FnMut(&T) -> bool>(&self, f: F) -> Option<I> {
        self.slice.iter().position(f).map(I::from_usize)
    }

    //ap last
    /// Return the the last element, if we are not empty.
    #[inline(always)]
    pub const fn last(&self) -> Option<&T> {
        self.slice.last()
    }

    //ap first
    /// Return the the first element, if we are not empty.
    #[inline]
    pub const fn first(&self) -> Option<&T> {
        self.slice.first()
    }

    //ap get
    /// Get a ref to the item at the provided index, or None for out of bounds.
    #[inline]
    pub fn get(&self, index: I) -> Option<&T> {
        index.opt_index().and_then(|index| self.slice.get(index))
    }

    //zz All done
}

//ip PartialEq for IndexedSlice
/*
impl<I: Idx, A, B> PartialEq<IndexSlice<I, [B]>> for IndexSlice<I, [A]>
where
    A: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &IndexSlice<I, [B]>) -> bool {
        PartialEq::eq(&self.raw, &other.raw)
    }
    #[inline]
    fn ne(&self, other: &IndexSlice<I, [B]>) -> bool {
        PartialEq::ne(&self.raw, &other.raw)
    }
}

impl<I: Idx, A: Eq> Eq for IndexSlice<I, [A]> {}

impl<I: Idx, A, B> PartialEq<[B]> for IndexSlice<I, [A]>
where
    A: PartialEq<B>,
{
    #[inline]
    fn eq(&self, other: &[B]) -> bool {
        PartialEq::eq(&self.raw, other)
    }
    #[inline]
    fn ne(&self, other: &[B]) -> bool {
        PartialEq::ne(&self.raw, other)
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a IndexSlice<I, [T]> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> slice::Iter<'a, T> {
        self.raw.iter()
    }
}

impl<I: Idx, T> Default for &IndexSlice<I, [T]> {
    #[inline]
    fn default() -> Self {
        IndexSlice::new(&[])
    }
}

impl<'a, I: Idx, T> From<&'a [T]> for &'a IndexSlice<I, [T]> {
    #[inline]
    fn from(a: &'a [T]) -> Self {
        IndexSlice::new(a)
    }
}

impl<I: Idx, A> AsRef<[A]> for IndexSlice<I, [A]> {
    #[inline]
    fn as_ref(&self) -> &[A] {
        &self.raw
    }
}

 */

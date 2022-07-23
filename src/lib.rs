#![cfg_attr(not(feature = "std"), no_std)]
//
#![cfg_attr(docsrs, feature(doc_cfg))]
//
#![doc = include_str!("../README.md")]
//
#![deny(anonymous_parameters)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unused)]
#![deny(unreachable_pub)]
//
// Warn (try not to do this)
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(variant_size_differences)]
#![warn(missing_docs)]
//
// Clippy
#![warn(clippy::pedantic)]

use bytes::Bytes;
use core::fmt::Display;
use core::iter::Enumerate;
use core::ops::{Range, RangeFrom, RangeFull, RangeTo};
use core::str::Utf8Error;
use nom::{
    AsBytes, Compare, InputIter, InputLength, InputTake, InputTakeAtPosition, Needed, Offset, Slice,
};

mod range_type;
pub use range_type::RangeType;

#[cfg(feature = "miette")]
#[cfg_attr(docsrs, doc(cfg(feature = "miette")))]
mod miette;

/// A wrapper around [`bytes::Bytes`] to be able to use it with [`nom`].
#[derive(Clone, Debug)]
pub struct NomBytes(Bytes, Option<RangeType<usize>>);

// Why the extra `Option<RangeType<usize>>`? Nom expects to be able to calculate
// offsets between two of its inputs, but `Bytes` has this optimization where if
// slicing results in an empty slice, it returns a new, empty `Bytes` rather than
// an empty slice of the existing `Bytes`. This causes problems down the line when
// nom asks for offsets between two inputs. Thus, in cases where slicing would
// result in an empty slice, we instead store the original `Bytes` plus the slice
// range itself, which we can use to hand out correct offsets.
//
// All the code here uses `bytes()` or `as_bytes()` for doing operations on the
// underlying bytes rather than accessing the "raw" `.0` field, because those two
// contain code that handles this custom slicing correctly, and thus we don't have
// to be careful anywhere else.
//
// Tried reporting this as unexpected/incorrect behavior, but it was said to be an
// intentional behavior:
// <https://github.com/tokio-rs/bytes/issues/557>

impl NomBytes {
    /// Creates a new `NomBytes` wrapping the provided [`Bytes`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    /// use nombytes::NomBytes;
    ///
    /// let b = Bytes::new();
    /// let nb = NomBytes::new(b);
    /// ```
    #[inline]
    pub fn new(bytes: Bytes) -> Self {
        Self(bytes, None)
    }

    /// Returns a string slice to the contents of the inner [`Bytes`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    /// use nombytes::NomBytes;
    ///
    /// let nb = NomBytes::new(Bytes::from("hello"));
    /// assert_eq!(nb.to_str(), "hello");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the [`Bytes`] slice is not UTF-8.
    #[inline]
    pub fn to_str(&self) -> &str {
        self.try_to_str().unwrap()
    }

    /// Returns a string slice to the contents of the inner [`Bytes`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    /// use nombytes::NomBytes;
    ///
    /// let nb = NomBytes::new(Bytes::from("hello"));
    /// assert_eq!(nb.try_to_str().unwrap(), "hello");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the [`Bytes`] slice is not UTF-8 with a description
    /// as to why the provided slice is not UTF-8.
    #[inline]
    pub fn try_to_str(&self) -> Result<&str, Utf8Error> {
        core::str::from_utf8(self.as_bytes())
    }

    #[doc = include_str!("to_bytes_doc.md")]
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    /// use nombytes::NomBytes;
    ///
    /// let nb = NomBytes::new(Bytes::from("hello"));
    /// let b = nb.to_bytes();
    /// assert_eq!(b.as_ref(), b"hello");
    /// ```
    #[inline]
    pub fn to_bytes(&self) -> Bytes {
        match self.1.as_ref() {
            Some(range) => self.0.slice(range.clone()),
            None => self.0.clone(),
        }
    }

    #[doc = include_str!("to_bytes_doc.md")]
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    /// use nombytes::NomBytes;
    ///
    /// let nb = NomBytes::new(Bytes::from("hello"));
    /// let b = nb.into_bytes();
    /// assert_eq!(b.as_ref(), b"hello");
    /// ```
    #[inline]
    pub fn into_bytes(self) -> Bytes {
        match self.1.as_ref() {
            Some(range) => self.0.slice(range.clone()),
            None => self.0,
        }
    }

    /// Returns the values from the inner representation of this type.
    ///
    /// See [`into_bytes`](Self::into_bytes) for an explanation of why this
    /// inner representation exists.
    // I dunno what anyone would use this for, but... might as well
    // offer it.
    pub fn into_raw(self) -> (Bytes, Option<RangeType<usize>>) {
        let Self(bytes, range_type) = self;
        (bytes, range_type)
    }

    /// Returns a new `NomBytes` using the raw values passed in. If these
    /// values represent something invalid, you'll likely see incorrect
    /// behavior or even panics. Regular usage should create values using
    /// [`new`](Self::new) instead.
    ///
    /// See [`into_bytes`](Self::into_bytes) for an explanation of why this
    /// inner representation exists.
    // I dunno what anyone would use this for, but... might as well
    // offer it.
    pub fn from_raw((bytes, range_type): (Bytes, Option<RangeType<usize>>)) -> Self {
        Self(bytes, range_type)
    }
}

impl AsBytes for NomBytes {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        match self.1.as_ref() {
            Some(range) => range.slice(self.0.as_ref()),
            None => self.0.as_ref(),
        }
    }
}

impl InputIter for NomBytes {
    type Item = u8;

    type Iter = Enumerate<Self::IterElem>;

    type IterElem = bytes::buf::IntoIter<Bytes>;

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.to_bytes().into_iter()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.to_bytes().iter().position(|b| predicate(*b))
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.to_bytes().len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.to_bytes().len()))
        }
    }
}

impl InputTake for NomBytes {
    #[inline]
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        let prefix = self.slice(..count);
        let suffix = self.slice(count..);
        (suffix, prefix)
    }
}

impl InputTakeAtPosition for NomBytes {
    type Item = <Self as InputIter>::Item;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.as_bytes().iter().position(|c| predicate(*c)) {
            Some(i) => Ok(self.take_split(i)),
            None => Err(nom::Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.as_bytes().iter().position(|c| predicate(*c)) {
            Some(0) => Err(nom::Err::Error(E::from_error_kind(self.clone(), e))),
            Some(i) => Ok(self.take_split(i)),
            None => Err(nom::Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.as_bytes().iter().position(|c| predicate(*c)) {
            Some(i) => Ok(self.take_split(i)),
            None => Ok(self.take_split(self.input_len())),
        }
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        let bytes = self.as_bytes();
        match bytes.iter().position(|c| predicate(*c)) {
            Some(0) => Err(nom::Err::Error(E::from_error_kind(self.clone(), e))),
            Some(i) => Ok(self.take_split(i)),
            None => {
                if bytes.is_empty() {
                    Err(nom::Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

impl InputLength for NomBytes {
    #[inline]
    fn input_len(&self) -> usize {
        self.as_bytes().len()
    }
}

macro_rules! nom_bytes_slice {
    ($range_ty:ty, $requirement:expr) => {
        impl Slice<$range_ty> for NomBytes {
            fn slice(&self, range: $range_ty) -> Self {
                let bytes = self.to_bytes();
                if bytes.is_empty() && $requirement(&range) {
                    return self.clone();
                }

                let slice = self.to_bytes().slice(range.clone());
                if slice.is_empty() {
                    NomBytes(bytes, Some(RangeType::from(range)))
                } else {
                    assert!(!slice.is_empty());
                    NomBytes(slice, None)
                }
            }
        }
    };
}

nom_bytes_slice!(Range<usize>, |r: &Range<usize>| r.start == 0 && r.end == 0);
nom_bytes_slice!(RangeTo<usize>, |r: &RangeTo<usize>| r.end == 0);
nom_bytes_slice!(RangeFrom<usize>, |r: &RangeFrom<usize>| r.start == 0);
nom_bytes_slice!(RangeFull, |_: &RangeFull| true);

impl Offset for NomBytes {
    #[inline]
    fn offset(&self, second: &Self) -> usize {
        self.as_bytes().offset(second.as_bytes())
    }
}

impl Compare<NomBytes> for NomBytes {
    #[inline]
    fn compare(&self, t: NomBytes) -> nom::CompareResult {
        self.as_bytes().compare(t.as_bytes())
    }

    #[inline]
    fn compare_no_case(&self, t: NomBytes) -> nom::CompareResult {
        self.as_bytes().compare_no_case(t.as_bytes())
    }
}

impl Compare<&'_ str> for NomBytes {
    #[inline]
    fn compare(&self, t: &str) -> nom::CompareResult {
        self.as_bytes().compare(t.as_bytes())
    }

    #[inline]
    fn compare_no_case(&self, t: &str) -> nom::CompareResult {
        self.as_bytes().compare_no_case(t.as_bytes())
    }
}

impl Display for NomBytes {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl From<&'_ str> for NomBytes {
    #[inline]
    fn from(string: &'_ str) -> Self {
        Self::from(string.as_bytes())
    }
}

impl From<&'_ [u8]> for NomBytes {
    #[inline]
    fn from(byte_slice: &'_ [u8]) -> Self {
        use bytes::{BufMut, BytesMut};

        let mut buf = BytesMut::with_capacity(byte_slice.len());
        buf.put(byte_slice);
        Self::new(buf.into())
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl From<String> for NomBytes {
    #[inline]
    fn from(string: String) -> Self {
        Self::new(Bytes::from(string))
    }
}

// We implement the eq/ord traits in terms of &[u8] since it's both
// cheap and easy:

impl PartialEq for NomBytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes().eq(other.as_bytes())
    }
}
impl Eq for NomBytes {}

impl PartialOrd for NomBytes {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}
impl Ord for NomBytes {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

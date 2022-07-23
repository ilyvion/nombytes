use core::ops::{Range, RangeBounds, RangeFrom, RangeFull, RangeTo};
use core::slice::SliceIndex;

/// An enum that can hold all the `Range*` types without being generic/trait
/// based. We need this type because `SliceIndex<T>` is implemented for the
/// individual `Range*` types rather than for the `RangeBounds<T>` trait that
/// we could've otherwise used.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RangeType<T> {
    /// Holds a [`RangeFull`] value
    RangeFull(RangeFull),
    /// Holds a [`RangeTo`] value
    RangeTo(RangeTo<T>),
    //RangeToInclusive(RangeToInclusive<T>),
    /// Holds a [`RangeFrom`] value
    RangeFrom(RangeFrom<T>),
    /// Holds a [`Range`] value
    Range(Range<T>),
    //RangeInclusive(RangeInclusive<T>),
}

impl<T: Clone> RangeType<T> {
    pub(crate) fn slice<'s, U>(&self, t: &'s [U]) -> &'s [U]
    where
        RangeTo<T>: SliceIndex<[U], Output = [U]>,
        //RangeToInclusive<T>: SliceIndex<[U], Output = [U]>,
        RangeFrom<T>: SliceIndex<[U], Output = [U]>,
        Range<T>: SliceIndex<[U], Output = [U]>,
        //RangeInclusive<T>: SliceIndex<[U], Output = [U]>,
    {
        match self.clone() {
            Self::RangeFull(_) => t,
            Self::RangeTo(r) => &t[r],
            //Self::RangeToInclusive(r) => &t[r],
            Self::RangeFrom(r) => &t[r],
            Self::Range(r) => &t[r],
            //Self::RangeInclusive(r) => &t[r],
        }
    }
}

impl<T> RangeBounds<T> for RangeType<T> {
    fn start_bound(&self) -> core::ops::Bound<&T> {
        match self {
            RangeType::RangeFull(r) => r.start_bound(),
            RangeType::RangeTo(r) => r.start_bound(),
            //RangeType::RangeToInclusive(r) => r.start_bound(),
            RangeType::RangeFrom(r) => r.start_bound(),
            RangeType::Range(r) => r.start_bound(),
            //RangeType::RangeInclusive(r) => r.start_bound(),
        }
    }

    fn end_bound(&self) -> core::ops::Bound<&T> {
        match self {
            RangeType::RangeFull(r) => r.end_bound(),
            RangeType::RangeTo(r) => r.end_bound(),
            //RangeType::RangeToInclusive(r) => r.end_bound(),
            RangeType::RangeFrom(r) => r.end_bound(),
            RangeType::Range(r) => r.end_bound(),
            //RangeType::RangeInclusive(r) => r.end_bound(),
        }
    }
}

macro_rules! range_to_from {
    ($range:ident) => {
        impl<T> From<$range<T>> for RangeType<T> {
            fn from(range: $range<T>) -> Self {
                Self::$range(range)
            }
        }
    };
}

impl<T> From<RangeFull> for RangeType<T> {
    fn from(range: RangeFull) -> Self {
        Self::RangeFull(range)
    }
}
range_to_from!(RangeTo);
//range_to_from!(RangeToInclusive);
range_to_from!(RangeFrom);
range_to_from!(Range);
//range_to_from!(RangeInclusive);

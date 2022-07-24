// We need this here because we can't set it on the output of the serde macros
#![allow(clippy::type_repetition_in_bounds)]

use core::ops::{Range, RangeBounds, RangeFrom, RangeFull, RangeTo};
use core::slice::SliceIndex;

/// An enum that can hold all the `Range*` types without being generic/trait
/// based. We need this type because `SliceIndex<T>` is implemented for the
/// individual `Range*` types rather than for the `RangeBounds<T>` trait that
/// we could've otherwise used.
//
// Because serde doesn't have good support for serializing the `Range*` types,
// we'll have to do it ourselves. See `RangeTypeSerialized` for implementation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        into = "range_type_serde::RangeTypeSerialized<T>",
        from = "range_type_serde::RangeTypeSerialized<T>",
        bound(
            serialize = "T: Clone, T: serde::Serialize",
            deserialize = "T: Clone, T: serde::de::Deserialize<'de>",
        )
    )
)]
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

#[cfg(feature = "serde")]
mod range_type_serde {
    use super::RangeType;
    use core::ops::{Bound, RangeBounds};

    #[derive(serde::Serialize, serde::Deserialize)]
    pub(crate) struct RangeTypeSerialized<T> {
        kind: u8,
        #[serde(skip_serializing_if = "Option::is_none")]
        start: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end: Option<T>,
    }

    impl<T: Clone> From<RangeType<T>> for RangeTypeSerialized<T> {
        fn from(r: RangeType<T>) -> Self {
            let (kind, start, end) = match &r {
                RangeType::RangeFull(r) => (0, r.start_bound(), r.end_bound()),
                RangeType::RangeTo(r) => (1, r.start_bound(), r.end_bound()),
                RangeType::RangeFrom(_) => (3, r.start_bound(), r.end_bound()),
                RangeType::Range(_) => (4, r.start_bound(), r.end_bound()),
            };
            let (start, end) = (bound_to_option(start), bound_to_option(end));

            Self { kind, start, end }
        }
    }

    impl<T> From<RangeTypeSerialized<T>> for RangeType<T> {
        fn from(rs: RangeTypeSerialized<T>) -> Self {
            let RangeTypeSerialized { kind, start, end } = rs;
            match kind {
                0 => RangeType::RangeFull(..),
                1 => RangeType::RangeTo(..end.unwrap()),
                3 => RangeType::RangeFrom(start.unwrap()..),
                4 => RangeType::Range(start.unwrap()..end.unwrap()),
                _ => unreachable!(),
            }
        }
    }

    #[inline]
    fn bound_to_option<T: Clone>(b: Bound<&'_ T>) -> Option<T> {
        match b {
            Bound::Unbounded => None,
            Bound::Included(x) | Bound::Excluded(x) => Some(x.clone()),
        }
    }

    #[test]
    fn ensure_roundtrip_works() {
        #[inline]
        fn roundtrip(rt: RangeType<usize>) {
            let rt_serialized = serde_json::to_string(&rt).unwrap();
            let rt_deserialized = serde_json::from_str(&rt_serialized).unwrap();

            assert_eq!(rt, rt_deserialized);
        }

        roundtrip(RangeType::RangeFull(..));
        roundtrip(RangeType::RangeTo(..42));
        roundtrip(RangeType::RangeFrom(42..));
        roundtrip(RangeType::Range(42..69));
    }
}

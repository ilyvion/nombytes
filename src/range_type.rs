// We need this here because we can't set it on the output of the serde macros
#![allow(clippy::type_repetition_in_bounds)]

use core::ops::{Index, Range, RangeBounds, RangeFrom, RangeFull, RangeTo};

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
        try_from = "range_type_serde::RangeTypeSerialized<T>",
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
    // XXX: Disable clippy false positive
    // <https://github.com/rust-lang/rust-clippy/issues/9076>
    #[allow(clippy::trait_duplication_in_bounds)]
    pub(crate) fn slice<'s, U>(&self, t: &'s U) -> &'s U
    where
        U: Index<RangeTo<T>, Output = U>
            + Index<RangeFrom<T>, Output = U>
            + Index<Range<T>, Output = U>
            + ?Sized,
    {
        match self.clone() {
            Self::RangeFull(_) => t,
            Self::RangeTo(r) => &t[r],
            Self::RangeFrom(r) => &t[r],
            Self::Range(r) => &t[r],
        }
    }
}

impl<T> RangeBounds<T> for RangeType<T> {
    fn start_bound(&self) -> core::ops::Bound<&T> {
        match self {
            RangeType::RangeFull(r) => r.start_bound(),
            RangeType::RangeTo(r) => r.start_bound(),
            RangeType::RangeFrom(r) => r.start_bound(),
            RangeType::Range(r) => r.start_bound(),
        }
    }

    fn end_bound(&self) -> core::ops::Bound<&T> {
        match self {
            RangeType::RangeFull(r) => r.end_bound(),
            RangeType::RangeTo(r) => r.end_bound(),
            RangeType::RangeFrom(r) => r.end_bound(),
            RangeType::Range(r) => r.end_bound(),
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
range_to_from!(RangeFrom);
range_to_from!(Range);

#[cfg(test)]
mod tests {
    use core::slice::SliceIndex;

    use super::RangeType;

    #[test]
    fn slice() {
        #[inline]
        fn assert_slice(input: &[u8], rt: RangeType<usize>, expected: &[u8]) {
            let output = rt.slice(input);

            assert_eq!(output, expected);
        }

        #[inline]
        fn assert_str(input: &str, rt: RangeType<usize>, expected: &str) {
            let output = rt.slice(input);

            assert_eq!(output, expected);
        }

        let input = b"hello, world!";
        assert_slice(input, RangeType::RangeFull(..), input);
        assert_slice(input, RangeType::RangeTo(..6), b"hello,");
        assert_slice(input, RangeType::RangeFrom(7..), b"world!");
        assert_slice(input, RangeType::Range(5..7), b", ");

        let input = "hello, world!";
        assert_str(input, RangeType::RangeFull(..), input);
        assert_str(input, RangeType::RangeTo(..6), "hello,");
        assert_str(input, RangeType::RangeFrom(7..), "world!");
        assert_str(input, RangeType::Range(5..7), ", ");
    }

    #[test]
    fn behaves_like_original_range() {
        #[inline]
        fn assert_identical_slice<T>(input: &str, range: T)
        where
            T: SliceIndex<str, Output = str> + Clone,
            RangeType<usize>: From<T>,
        {
            let sliced = &input[range.clone()];

            let range_type: RangeType<usize> = From::from(range);
            let output = range_type.slice(input);

            assert_eq!(sliced, output);
        }

        let input = "hello, world!";
        assert_identical_slice(input, ..);
        assert_identical_slice(input, ..6);
        assert_identical_slice(input, 7..);
        assert_identical_slice(input, 5..7);
    }
}

#[cfg(feature = "serde")]
mod range_type_serde {
    // A bug makes this seemingly both required and superfluous, but I can
    // only get it to consistently compile with it present, so it'll stay
    // for now.
    // <https://github.com/rust-lang/rust/issues/99637#issuecomment-1193619254>
    #[allow(clippy::useless_attribute)]
    #[allow(unused)]
    extern crate alloc;

    use super::RangeType;
    use core::ops::{Bound, RangeBounds};

    const RANGE_FULL_TAG: u8 = 0;
    const RANGE_TO_TAG: u8 = 1;
    const RANGE_FROM_TAG: u8 = 3;
    const RANGE_TAG: u8 = 4;

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
            let kind = match &r {
                RangeType::RangeFull(_) => RANGE_FULL_TAG,
                RangeType::RangeTo(_) => RANGE_TO_TAG,
                RangeType::RangeFrom(_) => RANGE_FROM_TAG,
                RangeType::Range(_) => RANGE_TAG,
            };
            let (start, end) = (
                bound_to_option(r.start_bound()),
                bound_to_option(r.end_bound()),
            );

            Self { kind, start, end }
        }
    }

    impl<T> TryFrom<RangeTypeSerialized<T>> for RangeType<T> {
        type Error = RangeTypeDeserializationError;
        fn try_from(rs: RangeTypeSerialized<T>) -> Result<Self, Self::Error> {
            let RangeTypeSerialized { kind, start, end } = rs;
            Ok(match kind {
                RANGE_FULL_TAG => RangeType::RangeFull(..),
                RANGE_TO_TAG => RangeType::RangeTo(..end.unwrap()),
                RANGE_FROM_TAG => RangeType::RangeFrom(start.unwrap()..),
                RANGE_TAG => RangeType::Range(start.unwrap()..end.unwrap()),
                x => return Err(RangeTypeDeserializationError(x)),
            })
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct RangeTypeDeserializationError(u8);
    impl core::fmt::Display for RangeTypeDeserializationError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "Invalid RangeType `kind` value encountered: {}", self.0)
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

    #[test]
    fn trigger_error() {
        //extern crate alloc;

        const INVALID_REPRESENTATION: &str = r#"{"kind": 42}"#;
        let invalid_deserialized_error: Result<RangeType<usize>, _> =
            serde_json::from_str(INVALID_REPRESENTATION);

        assert!(invalid_deserialized_error.is_err());
        assert_eq!(
            &alloc::format!("{}", invalid_deserialized_error.unwrap_err()),
            "Invalid RangeType `kind` value encountered: 42"
        );
    }
}

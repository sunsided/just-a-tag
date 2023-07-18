//! # Just a tag
//!
//! This crate contains the [`Tag`] type, an [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035)
//! DNS label compatible string, with parsing [`FromStr`] and optional [serde](https://serde.rs/) support.
//!
//! ## Tag examples
//!
//! ```
//! # use justatag::Tag;
//! assert_eq!(Tag::new("some-tag"), "some-tag");
//! assert_eq!(Tag::from_str("some-tag").unwrap(), "some-tag");
//! assert!(Tag::from_str("invalid-").is_err());
//! ```
//!
//! ## Unions of tags
//!
//! A bit untrue to the crate's name, it also provides the [`TagUnion`] type, which represents
//! (unsurprisingly, this time) a union of tags.
//!
//! ```
//! use std::collections::HashSet;
//! use justatag::{MatchesAnyTagUnion, Tag, TagUnion};
//!
//! let union = TagUnion::from_str("foo").unwrap();
//! assert!(union.contains(&Tag::new("foo")));
//! assert_eq!(union.len(), 1);
//!
//! let union = TagUnion::from_str("foo+bar").unwrap();
//! assert!(union.contains(&Tag::new("foo")));
//! assert!(union.contains(&Tag::new("bar")));
//! assert_eq!(union.len(), 2);
//!
//! // TagUnions are particularly interesting when bundled up.
//! let unions = vec![
//!     TagUnion::from_str("bar+baz").unwrap(),
//!     TagUnion::from_str("foo").unwrap()
//! ];
//!
//! // foo matches
//! let set_1 = HashSet::from_iter([Tag::new("foo"), Tag::new("bar")]);
//! assert!(unions.matches_set(&set_1));
//!
//! // bar+baz matches
//! let set_2 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar"), Tag::new("baz")]);
//! assert!(unions.matches_set(&set_2));
//!
//! // none match
//! let set_3 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar")]);
//! assert!(!unions.matches_set(&set_3));
//! ```

// SPDX-FileCopyrightText: Copyright 2023 Markus Mayer
// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileType: SOURCE

// Only enable the `doc_cfg` feature when the `docsrs` configuration attribute is defined.
#![cfg_attr(docsrs, feature(doc_cfg))]

mod tag_union;

#[cfg_attr(feature = "unsafe", allow(unsafe_code))]
#[cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

pub use tag_union::{MatchesAnyTagUnion, TagUnion, TagUnionFromStringError};

/// A tag name.
///
/// Tag names [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035) DNS label compatible,
/// in other words they must
///
/// - not be longer than 63 characters,,
/// - only lowercase alphanumeric characters or '-',
/// - start with an alphabetic character, and
/// - end with an alphanumeric character.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Tag(String);

impl Tag {
    /// An empty tag.
    pub const EMPTY: Tag = Tag(String::new());

    /// The maximum length of a tag.
    pub const MAX_LEN: usize = 63;

    /// Constructs a new tag.
    ///
    /// ## Panics
    ///
    /// This method panics if the input is not a valid tag. If you want to avoid a panic,
    /// use [`Tag::from_str`](Self::from_str) instead.
    ///
    /// ## Example
    ///
    /// ```
    /// use justatag::Tag;
    /// assert_eq!(Tag::new("foo"), "foo");
    /// ```
    pub fn new<V: AsRef<str>>(value: V) -> Self {
        value.as_ref().parse().expect("invalid input")
    }

    /// Constructs a new tag without checking for validity.
    ///
    /// ## Example
    ///
    /// ```
    /// # use justatag::Tag;
    /// /// // Constructs a Tag without verifying the input.
    /// assert_eq!(unsafe { Tag::new_unchecked("foo") }, "foo");
    /// assert_eq!(unsafe { Tag::new_unchecked("@") }, "@"); // NOTE: invalid input
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "unsafe")))]
    #[cfg(feature = "unsafe")]
    pub unsafe fn new_unchecked<V: Into<String>>(value: V) -> Self {
        Self(value.into())
    }

    /// Parses a [`Tag`] from a string-like value.
    ///
    /// ```
    /// # use justatag::Tag;
    /// assert_eq!(Tag::from_str("some-tag").unwrap(), "some-tag");
    /// assert!(Tag::from_str("invalid-").is_err());
    /// ```
    pub fn from_str<S: AsRef<str>>(value: S) -> Result<Self, TagFromStringError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Ok(Tag::EMPTY.clone());
        }

        if value.len() > Tag::MAX_LEN {
            return Err(TagFromStringError::LimitExceeded(value.len()));
        }

        let mut chars = value.chars();
        let first = chars.next().expect("tag is not empty");
        if !first.is_ascii_lowercase() {
            return Err(TagFromStringError::MustStartAlphabetic(first));
        }

        let mut previous = first;
        while let Some(c) = chars.next() {
            if !c.is_ascii_digit() && !c.is_ascii_lowercase() && c != '-' {
                return Err(TagFromStringError::InvalidCharacter(c));
            }

            previous = c;
        }

        if !previous.is_ascii_lowercase() {
            return Err(TagFromStringError::MustEndAlphanumeric(previous));
        }

        Ok(Self(value.into()))
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Tag {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl PartialEq<str> for Tag {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<&str> for Tag {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.0.eq(other)
    }
}

impl FromStr for Tag {
    type Err = TagFromStringError;

    #[inline(always)]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Tag::from_str(value)
    }
}

impl TryFrom<&str> for Tag {
    type Error = TagFromStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for Tag {
    type Error = TagFromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<&String> for Tag {
    type Error = TagFromStringError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum TagFromStringError {
    #[error("Tag name must begin with a lowercase alphabetic character, got '{0}'")]
    MustStartAlphabetic(char),
    #[error("Tag name must end with a lowercase alphanumeric character, got '{0}'")]
    MustEndAlphanumeric(char),
    #[error("Tag name must only contain lowercase alphanumeric characters or '-', got '{0}'")]
    InvalidCharacter(char),
    #[error("Tag name must be not longer than 63 characters, got '{0}'")]
    LimitExceeded(usize),
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tag = String::deserialize(deserializer)?;
        match Tag::from_str(&tag) {
            Ok(tag) => Ok(tag),
            Err(e) => Err(de::Error::custom(e)),
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial() {
        assert_eq!(Tag::from_str("test").unwrap(), "test");
        assert_eq!(Tag::from_str("test-case").unwrap(), "test-case");
        assert_eq!(Tag::from_str("test---12e").unwrap(), "test---12e");
        assert!(
            Tag::from_str("a123456789a123456789a123456789a123456789a123456789a12345678901a")
                .is_ok()
        );
    }

    #[test]
    fn test_invalid() {
        assert!(Tag::from_str("1").is_err());
        assert!(Tag::from_str("-").is_err());
        assert!(Tag::from_str("a-").is_err());
        assert!(Tag::from_str("a1").is_err());
        assert!(Tag::from_str("a_b_c").is_err());
        assert!(
            Tag::from_str("a123456789a123456789a123456789a123456789a123456789a123456789012a")
                .is_err()
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_de_trivial() {
        assert_eq!(serde_json::from_str::<Tag>(r#""test""#).unwrap(), "test");
        assert_eq!(
            serde_json::from_str::<Tag>(r#""test-case""#).unwrap(),
            "test-case"
        );
        assert_eq!(
            serde_json::from_str::<Tag>(r#""test---12e""#).unwrap(),
            "test---12e"
        );
        assert!(serde_json::from_str::<Tag>(
            r#""a123456789a123456789a123456789a123456789a123456789a12345678901a""#
        )
        .is_ok());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_de_invalid() {
        assert!(serde_json::from_str::<Tag>(r#""1""#).is_err());
        assert!(serde_json::from_str::<Tag>(r#""-""#).is_err());
        assert!(serde_json::from_str::<Tag>(r#""a-""#).is_err());
        assert!(serde_json::from_str::<Tag>(r#""a1""#).is_err());
        assert!(serde_json::from_str::<Tag>(r#""a_b_c""#).is_err());
        assert!(serde_json::from_str::<Tag>(
            r#""a123456789a123456789a123456789a123456789a123456789a123456789012a""#
        )
        .is_err());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_ser_invalid() {
        let json = serde_json::to_string(&Tag::new("foo")).unwrap();
        assert_eq!(json, r#""foo""#);
    }
}

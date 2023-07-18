//! Provides the [`TagUnion`] type, a union of tags.

// SPDX-FileCopyrightText: Copyright 2023 Markus Mayer
// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileType: SOURCE

use crate::{Tag, TagFromStringError};
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;
use std::str::FromStr;

/// A tag union, e.g. `foo` or `foo+bar+baz` (i.e. `foo` _and_ `bar` _and_ `baz`).
///
/// ```
/// use justatag::{Tag, TagUnion};
///
/// let union = TagUnion::from_str("foo").unwrap();
/// assert!(union.contains(&Tag::new("foo")));
/// assert_eq!(union.len(), 1);
///
/// let union = TagUnion::from_str("foo+bar").unwrap();
/// assert!(union.contains(&Tag::new("foo")));
/// assert!(union.contains(&Tag::new("bar")));
/// assert_eq!(union.len(), 2);
///
/// assert!(TagUnion::from_str("foo bar").is_err());
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct TagUnion(HashSet<Tag>);

impl TagUnion {
    /// Returns `true` if this tag union matches the value presented in the set.
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use justatag::{MatchesAnyTagUnion, Tag, TagUnion};
    ///
    /// let unions = vec![
    ///     TagUnion::from_str("foo").unwrap(),
    ///     TagUnion::from_str("bar+baz").unwrap()
    /// ];
    ///
    /// // foo, and bar+baz matches
    /// let set_1 = HashSet::from_iter([Tag::new("foo"), Tag::new("bar"), Tag::new("baz")]);
    /// assert!(unions.matches_set(&set_1));
    ///
    /// // bar+baz matches
    /// let set_2 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar"), Tag::new("baz")]);
    /// assert!(unions.matches_set(&set_2));
    ///
    /// // foo matches
    /// let set_3 = HashSet::from_iter([Tag::new("foo"), Tag::new("bar")]);
    /// assert!(unions.matches_set(&set_3));
    ///
    /// // none match
    /// let set_4 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar")]);
    /// assert!(!unions.matches_set(&set_4));
    /// ```
    pub fn matches_set(&self, values: &HashSet<Tag>) -> bool {
        self.0.is_subset(values)
    }

    /// Inserts a tag into this union.
    /// Returns whether the tag was inserted; that is:
    ///
    /// * If the tag was not previously inserted, `true` is returned,
    /// * If the tag was previously inserted, `false` is returned.
    pub fn insert(&mut self, tag: Tag) -> bool {
        self.0.insert(tag)
    }

    /// Removes a tag from this union.
    /// Returns whether the tag was removed; that is:
    ///
    /// * If the tag was previously inserted, `true` is returned,
    /// * If the tag was not previously inserted, `false` is returned.
    pub fn remove<T: Borrow<Tag>>(&mut self, tag: T) -> bool {
        self.0.remove(tag.borrow())
    }

    /// Returns whether this union contains the specified tag. That is:
    ///
    /// * If the tag was previously inserted, `true` is returned,
    /// * If the tag was not previously inserted, `false` is returned.
    pub fn contains<T: Borrow<Tag>>(&self, tag: &T) -> bool {
        self.0.contains(tag.borrow())
    }

    /// Attempts to parse a [`TagUnion`] from a string-like input.
    pub fn from_str<S: AsRef<str>>(value: S) -> Result<TagUnion, TagUnionFromStringError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Ok(TagUnion::default());
        }

        let parts = value.split('+');
        let names: HashSet<String> = parts
            .filter(|&c| !c.contains('+'))
            .filter(|&c| !c.is_empty())
            .map(|c| c.into())
            .collect();

        if names.is_empty() {
            return Ok(TagUnion::default());
        }

        let mut tags = HashSet::new();
        for name in names.into_iter() {
            tags.insert(Tag::from_str(&name)?);
        }

        Ok(Self(tags))
    }
}

/// Implements
pub trait MatchesAnyTagUnion {
    /// Returns `true` if this tag union matches the value presented in the set.
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use justatag::{MatchesAnyTagUnion, Tag, TagUnion};
    ///
    /// let unions = vec![
    ///     TagUnion::from_str("foo").unwrap(),
    ///     TagUnion::from_str("bar+baz").unwrap()
    /// ];
    ///
    /// // foo, and bar+baz matches
    /// let set_1 = HashSet::from_iter([Tag::new("foo"), Tag::new("bar"), Tag::new("baz")]);
    /// assert!(unions.matches_set(&set_1));
    ///
    /// // bar+baz matches
    /// let set_2 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar"), Tag::new("baz")]);
    /// assert!(unions.matches_set(&set_2));
    ///
    /// // foo matches
    /// let set_3 = HashSet::from_iter([Tag::new("foo"), Tag::new("bar")]);
    /// assert!(unions.matches_set(&set_3));
    ///
    /// // none match
    /// let set_4 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar")]);
    /// assert!(!unions.matches_set(&set_4));
    /// ```
    fn matches_set(&self, values: &HashSet<Tag>) -> bool;
}

impl MatchesAnyTagUnion for Vec<TagUnion> {
    fn matches_set(&self, values: &HashSet<Tag>) -> bool {
        self.iter().any(|s| s.matches_set(&values))
    }
}

impl Deref for TagUnion {
    type Target = HashSet<Tag>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for TagUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut vec = Vec::from_iter(self.0.iter());
        vec.sort();
        for tag in vec {
            tag.hash(state);
        }
    }
}

impl FromIterator<Tag> for TagUnion {
    fn from_iter<T: IntoIterator<Item = Tag>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl FromStr for TagUnion {
    type Err = TagUnionFromStringError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        TagUnion::from_str(value)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for TagUnion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;
        match TagUnion::from_str(&input) {
            Ok(tags) => Ok(tags),
            Err(e) => Err(de::Error::custom(e)),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TagUnionFromStringError {
    InvalidTag(TagFromStringError),
}

impl Display for TagUnionFromStringError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TagUnionFromStringError::InvalidTag(e) => write!(f, "Invalid tag: {e}"),
        }
    }
}

impl From<TagFromStringError> for TagUnionFromStringError {
    fn from(value: TagFromStringError) -> Self {
        Self::InvalidTag(value)
    }
}

impl Error for TagUnionFromStringError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let tags: TagUnion = TagUnion::from_str("").unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_add_remove() {
        let mut tags = TagUnion::from_str(r#"foo"#).unwrap();
        assert!(tags.contains(&Tag::new("foo")));
        assert_eq!(tags.len(), 1);

        tags.insert(Tag::new("bar"));
        assert_eq!(tags.len(), 2);
        tags.contains(&Tag::new("bar"));

        tags.remove(&Tag::new("foo"));
        assert!(!tags.contains(&Tag::new("foo")));
        assert_eq!(tags.len(), 1);

        tags.remove(&Tag::new("bar"));
        assert_eq!(tags.len(), 0);
        assert!(tags.is_empty());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_trivial() {
        let tags: TagUnion = serde_json::from_str(r#""foo""#).unwrap();
        assert!(tags.contains(&Tag::new("foo")));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_complex() {
        let tags: TagUnion = serde_json::from_str(r#""foo+bar+++baz++""#).unwrap();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&Tag::new("foo")));
        assert!(tags.contains(&Tag::new("bar")));
        assert!(tags.contains(&Tag::new("baz")));
    }

    #[test]
    fn test_invalid() {
        let tags = TagUnion::from_str(r#"foo+#baz"#);
        assert_eq!(
            tags,
            Err(TagUnionFromStringError::InvalidTag(
                crate::TagFromStringError::MustStartAlphabetic('#')
            ))
        );
    }

    #[test]
    fn test_matches() {
        let selections = vec![
            TagUnion::from_str("foo+bar").unwrap(),
            TagUnion::from_str("baz").unwrap(),
        ];

        // foo+bar are present, so is baz
        assert!(selections.matches_set(&HashSet::from_iter([
            Tag::new("foo"),
            Tag::new("bar"),
            Tag::new("baz"),
        ])));

        // baz is present
        assert!(selections.matches_set(&HashSet::from_iter([Tag::new("baz"),])));

        // foo+bar are present
        assert!(selections.matches_set(&HashSet::from_iter([Tag::new("foo"), Tag::new("bar"),])));

        // baz present
        assert!(selections.matches_set(&HashSet::from_iter([Tag::new("foo"), Tag::new("baz"),])));

        // neither foo+bar, nor baz are present.
        assert!(!selections.matches_set(&HashSet::from_iter([Tag::new("foo"), Tag::new("bang"),])));
    }
}

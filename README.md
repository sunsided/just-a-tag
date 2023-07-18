# Just a tag.

This crate contains the `Tag` type, an [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035)
DNS label compatible string, with parsing `FromStr` and optional [serde](https://serde.rs/) support.


## Tag examples

```rust
use justatag::Tag;

fn tags() {
    assert_eq!(Tag::new("some-tag"), "some-tag");
    assert_eq!(Tag::from_str("some-tag").unwrap(), "some-tag");
    assert!(Tag::from_str("invalid-").is_err());
}
```

## Unions of tags

A bit untrue to the crate's name, it also provides the [`TagUnion`] type, which represents
(unsurprisingly, this time) a union of tags.

```rust
use std::collections::HashSet;
use justatag::{MatchesAnyTagUnion, Tag, TagUnion};

fn tag_unions() {
    let union = TagUnion::from_str("foo").unwrap();
    assert!(union.contains(&Tag::new("foo")));
    assert_eq!(union.len(), 1);

    let union = TagUnion::from_str("foo+bar").unwrap();
    assert!(union.contains(&Tag::new("foo")));
    assert!(union.contains(&Tag::new("bar")));
    assert_eq!(union.len(), 2);

    // TagUnions are particularly interesting when bundled up.
    let unions = vec![
        TagUnion::from_str("bar+baz").unwrap(),
        TagUnion::from_str("foo").unwrap()
    ];

    // foo matches
    let set_1 = HashSet::from_iter([Tag::new("foo"), Tag::new("bar")]);
    assert!(unions.matches_set(&set_1));

    // bar+baz matches
    let set_2 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar"), Tag::new("baz")]);
    assert!(unions.matches_set(&set_2));

    // none match
    let set_3 = HashSet::from_iter([Tag::new("fubar"), Tag::new("bar")]);
    assert!(!unions.matches_set(&set_3));
}
```

# Just a tag.

This crate contains the `Tag` type, an [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035)
DNS label compatible string, with parsing `FromStr` and optional [serde](https://serde.rs/) support.


```rust
#[test]
fn test() {
    assert_eq!(Tag::new("some-tag"), "some-tag");
    assert_eq!(Tag::from_str("some-tag").unwrap(), "some-tag");
    assert!(Tag::from_str("invalid-").is_err());
}
```

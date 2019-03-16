# rfind_url

[![Build Status](https://travis-ci.com/chrisduerr/rfind_url.svg?branch=master)](https://travis-ci.com/chrisduerr/rfind_url)
[![crates.io](https://meritbadge.herokuapp.com/rfind_url)](https://crates.io/crates/rfind_url)

This crate provides a parser to search a string for URLs **in reverse order**.

All functionality is handled by the
[`Parser`](https://docs.rs/rfind_url/*/rfind_url/struct.Parser.html) struct which takes
[`chars`](https://doc.rust-lang.org/std/primitive.char.html) as input.

# Examples

Text can be fed into the parser in reverse order:

```
use rfind_url::Parser;

let mut parser = Parser::new();

for c in "There is no URL here.".chars().rev() {
    assert_eq!(parser.advance(c), None);
}
```

The parser returns the length of the URL as soon as the last character of the URL is pushed into
it. Otherwise it will return
[`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None):

```
use rfind_url::Parser;

let mut parser = Parser::new();

// Parser did not find any URLs
assert_eq!(parser.advance(' '), None);

// URLs are only returned once they are complete
for c in "ttps://example.org".chars().rev() {
    assert_eq!(parser.advance(c), None);
}

// Parser has detected a URL spanning the last 19 characters
assert_eq!(parser.advance('h'), Some(19));
```

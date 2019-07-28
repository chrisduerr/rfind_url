# Reverse Find URL

[![Build Status](https://travis-ci.org/chrisduerr/rfind_url.svg?branch=master)](https://travis-ci.org/chrisduerr/rfind_url)
[![crates.io](https://meritbadge.herokuapp.com/rfind_url)](https://crates.io/crates/rfind_url)

This crate provides a parser to search a string for URLs **in reverse order**.

All functionality is handled by the
[`Parser`](https://docs.rs/rfind_url/*/rfind_url/struct.Parser.html) struct which takes
[`chars`](https://doc.rust-lang.org/std/primitive.char.html) as input.

# Examples

Text can be fed into the parser in reverse order:

```
use rfind_url::{Parser, ParserState};

let mut parser = Parser::new();

for c in "There_is_no_URL_here".chars().rev() {
    assert_eq!(parser.advance(c), ParserState::MaybeUrl);
}
```

The parser returns the length of the URL as soon as the last character of the URL is pushed into
it. Otherwise it will return
[`None`](https://doc.rust-lang.org/std/option/enum.Option.html#variant.None):

```
use rfind_url::{Parser, ParserState};

let mut parser = Parser::new();

// Parser guarantees there's currently no active URL
assert_eq!(parser.advance(' '), ParserState::NoUrl);

// URLs are only returned once they are complete
for c in "ttps://example.org".chars().rev() {
    assert_eq!(parser.advance(c), ParserState::MaybeUrl);
}

// Parser has detected a URL spanning the last 19 characters
assert_eq!(parser.advance('h'), ParserState::Url(19));
```

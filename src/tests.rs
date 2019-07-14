use std::collections::HashMap;

use crate::{Parser, State, SCHEMES};

#[test]
fn no_scheme_conflicts() {
    for scheme in &SCHEMES {
        for other_scheme in SCHEMES.iter().filter(|&s| s != scheme) {
            assert!(!scheme.ends_with(other_scheme),);
        }
    }
}

#[test]
fn index_lens() {
    assert_eq!(SCHEMES.len(), Parser::new().scheme_indices.len());
}

#[test]
fn boundaries() {
    assert_eq!(max_len("before https://example.org after"), Some(19));

    assert_eq!(position("before https://example.org after"), (7, 6));
    assert_eq!(position("before https://example.org"), (7, 0));
    assert_eq!(position("https://example.org after"), (0, 6));
}

#[test]
fn start() {
    assert_eq!(max_len("https://example.org/test\u{00}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\u{1F}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\u{7F}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\u{9F}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\ting"), Some(24));
    assert_eq!(max_len("https://example.org/test ing"), Some(24));
    assert_eq!(max_len("https://example.org/test?ing"), Some(28));
    assert_eq!(max_len("https://example.org.,;:(!/?"), Some(19));
    assert_eq!(max_len("https://example.org/"), Some(20));
}

#[test]
fn end() {
    assert_eq!(max_len("complicated:https://example.org"), Some(19));
    assert_eq!(max_len("\u{2502}https://example.org"), Some(19));
    assert_eq!(max_len("test.https://example.org"), Some(19));
    assert_eq!(max_len("https://sub.example.org"), Some(23));
    assert_eq!(max_len(",https://example.org"), Some(19));
}

#[test]
fn url_unicode() {
    assert_eq!(max_len("https://xn--example-2b07f.org"), Some(29));
    assert_eq!(max_len("https://example.org/\u{2008A}"), Some(21));
    assert_eq!(max_len("https://example.org/\u{f17c}"), Some(21));
    assert_eq!(max_len("https://üñîçøðé.com/ä"), Some(21));
}

#[test]
fn url_schemes() {
    assert_eq!(max_len("invalidscheme://example.org"), None);
    assert_eq!(max_len("mailto://example.org"), Some(20));
    assert_eq!(max_len("https://example.org"), Some(19));
    assert_eq!(max_len("http://example.org"), Some(18));
    assert_eq!(max_len("news://example.org"), Some(18));
    assert_eq!(max_len("file://example.org"), Some(18));
    assert_eq!(max_len("git://example.org"), Some(17));
    assert_eq!(max_len("ssh://example.org"), Some(17));
    assert_eq!(max_len("ftp://example.org"), Some(17));
}

#[test]
fn url_matching_chars() {
    assert_eq!(max_len("(https://example.org/test(ing)/?)"), Some(30));
    assert_eq!(max_len("(https://example.org/test(ing))"), Some(29));
    assert_eq!(max_len("https://example.org/test(ing)"), Some(29));
    assert_eq!(max_len("((https://example.org))"), Some(19));
    assert_eq!(max_len(")https://example.org("), Some(19));
    assert_eq!(max_len("https://example.org)"), Some(19));
    assert_eq!(max_len("https://example.org("), Some(19));

    assert_eq!(max_len("https://[2001:db8:a0b:12f0::1]:80"), Some(33));
    assert_eq!(max_len("([(https://example.org/test(ing))])"), Some(29));
    assert_eq!(max_len("https://example.org/]()"), Some(20));
    assert_eq!(max_len("[https://example.org]"), Some(19));

    assert_eq!(max_len("'https://example.org/test'ing'''"), Some(29));
    assert_eq!(max_len("https://example.org/test'ing'"), Some(29));
    assert_eq!(max_len("'https://example.org'"), Some(19));
    assert_eq!(max_len("'https://example.org"), Some(19));
    assert_eq!(max_len("https://example.org'"), Some(19));
}

#[test]
fn markdown() {
    let input = "[test](https://example.org)";
    let mut result_map = HashMap::new();
    result_map.insert(19, Some(19));
    exact_url_match(input, result_map);

    let input = "[https://example.org](test)";
    let mut result_map = HashMap::new();
    result_map.insert(25, Some(19));
    exact_url_match(input, result_map);

    let input = "[https://example.org](https://example.org/longer)";
    let mut result_map = HashMap::new();
    result_map.insert(26, Some(26));
    result_map.insert(47, Some(19));
    exact_url_match(input, result_map);
}

#[test]
fn multiple_urls() {
    let input = "test https://example.org illegal://example.com https://example.com/test 123";
    let mut result_map = HashMap::new();
    result_map.insert(27, Some(24));
    result_map.insert(69, Some(19));
    exact_url_match(input, result_map);
}

#[test]
fn reset_on_match() {
    let mut parser = Parser::new();

    for c in "https://example.org".chars().rev() {
        parser.advance(c);
    }

    assert_eq!(parser.state, State::Default);
}

fn exact_url_match(input: &str, result_map: HashMap<usize, Option<u16>>) {
    let mut parser = Parser::new();

    for (i, c) in input.chars().rev().enumerate() {
        let result = parser.advance(c);

        if let Some(expected) = result_map.get(&i) {
            assert_eq!(&result, expected);
        } else {
            assert_eq!(result, None);
        }
    }
}

fn max_len(input: &str) -> Option<u16> {
    let mut parser = Parser::new();
    let mut url_len = None;

    for c in input.chars().rev() {
        if let Some(len) = parser.advance(c) {
            url_len = Some(len);
        }
    }

    url_len
}

fn position(input: &str) -> (usize, usize) {
    let mut parser = Parser::new();
    let mut position_right = 0usize;
    let mut position_left = 0usize;
    let mut url_len = None;

    for c in input.chars().rev() {
        if url_len.is_some() {
            position_left += 1;
        } else {
            position_right += 1;
        }

        if let Some(len) = parser.advance(c) {
            url_len = Some(len);
        }
    }

    if let Some(url_len) = url_len {
        position_right = position_right.saturating_sub(url_len as usize);
    }

    (position_left, position_right)
}

#[cfg(feature = "bench")]
mod bench {
    extern crate std;
    extern crate test;

    use std::string::String;

    use crate::Parser;

    #[bench]
    fn library(b: &mut test::Bencher) {
        let mut input = String::new();
        for i in 0..10_000 {
            if i % 1_000 == 0 {
                input.push_str("https://example.org");
            } else {
                input.push_str(" test ");
            }
        }

        b.iter(|| {
            let mut parser = Parser::new();
            for c in input.chars().rev() {
                if let Some(url_len) = parser.advance(c) {
                    test::black_box(url_len);
                }
            }
        });
    }

    #[bench]
    fn lower_bound(b: &mut test::Bencher) {
        let mut input = String::new();
        for i in 0..10_000 {
            if i % 1_000 == 0 {
                input.push_str("https://example.org");
            } else {
                input.push_str(" test ");
            }
        }

        b.iter(|| {
            for c in input.chars().rev() {
                test::black_box(c);
            }
        });
    }
}

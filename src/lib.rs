//! This crate provides a parser to search a string for URLs **in reverse order**.
//!
//! All functionality is handled by the [`Parser`] struct which takes [`chars`] as input.
//!
//! # Examples
//!
//! Text can be fed into the parser in reverse order:
//!
//! ```
//! use rfind_url::{Parser, ParserState};
//!
//! let mut parser = Parser::new();
//!
//! for c in "There_is_no_URL_here".chars().rev() {
//!     assert_eq!(parser.advance(c), ParserState::MaybeUrl);
//! }
//! ```
//!
//! The parser returns the length of the URL as soon as the last character of the URL is pushed
//! into it. If it can be guaranteed that the current character is not part of a URL,
//! [`ParserState::NoUrl`] will be returned. Otherwise the response will be
//! [`ParserState::MaybeUrl`].
//!
//! ```
//! use rfind_url::{Parser, ParserState};
//!
//! let mut parser = Parser::new();
//!
//! // Parser guarantees there's currently no active URL
//! assert_eq!(parser.advance(' '), ParserState::NoUrl);
//!
//! // URLs are only returned once they are complete
//! for c in "ttps://example.org".chars().rev() {
//!     assert_eq!(parser.advance(c), ParserState::MaybeUrl);
//! }
//!
//! // Parser has detected a URL spanning the last 19 characters
//! assert_eq!(parser.advance('h'), ParserState::Url(19));
//! ```
//!
//! [`chars`]: https://doc.rust-lang.org/std/primitive.char.html
//! [`ParserState::MaybeUrl`]: enum.ParserState.html#variant.MaybeUrl
//! [`ParserState::NoUrl`]: enum.ParserState.html#variant.NoUrl
//! [`Parser`]: struct.Parser.html

#![cfg_attr(all(test, feature = "bench"), feature(test))]

#[cfg(test)]
mod tests;

/// Allowed URL schemes.
pub(crate) const SCHEMES: [&str; 8] =
    ["http", "https", "mailto", "news", "file", "git", "ssh", "ftp"];

const SURROUND_CHARACTERS: [SurroundCharacter; 3] = [
    SurroundCharacter::Bracket('(', ')'),
    SurroundCharacter::Bracket('[', ']'),
    SurroundCharacter::Quote('\''),
];

/// Internal URL parser states.
#[derive(Debug, PartialEq)]
enum State {
    Default,
    Path,
    SchemeFirstSlash,
    SchemeSecondSlash,
    Scheme,
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        State::Default
    }
}

/// Current parser state.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParserState {
    /// Parser finished parsing a URL with the defined length.
    Url(usize),
    /// Parser might currently be inside a URL.
    MaybeUrl,
    /// Parser is not inside a URL.
    NoUrl,
}

/// State machine for finding URLs.
///
/// The URL parser takes characters of a string **in reverse order** and returns the length of the
/// URL whenever finding one.
#[derive(Default)]
pub struct Parser {
    pub(crate) scheme_indices: [u8; 8],
    pub(crate) state: State,

    surround_states: Vec<(char, usize)>,
    len: usize,
}

impl Parser {
    /// Creates a new URL parser.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfind_url::Parser;
    ///
    /// let mut parser = Parser::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances the parser by one character.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfind_url::{Parser, ParserState};
    ///
    /// let mut parser = Parser::new();
    ///
    /// // Current character is not part of a URL
    /// assert_eq!(parser.advance(' '), ParserState::NoUrl);
    ///
    /// // URLs are only returned once they are complete
    /// for c in "ttps://example.org".chars().rev() {
    ///     assert_eq!(parser.advance(c), ParserState::MaybeUrl);
    /// }
    ///
    /// // Parser has detected a URL spanning the last 19 characters
    /// assert_eq!(parser.advance('h'), ParserState::Url(19));
    /// ```
    #[inline]
    pub fn advance(&mut self, c: char) -> ParserState {
        self.len += 1;

        if is_illegal(c) {
            self.reset();
            return ParserState::NoUrl;
        }

        // Check if this is a matching opening character
        let m = self.surround_states.iter().enumerate().rfind(|s| (s.1).0 == c);

        // Remove match to permit this surrounding, if surround is not empty
        if let Some((index, elem)) = m {
            if elem.1 + 1 < self.len {
                self.surround_states.remove(index);
                return ParserState::MaybeUrl;
            }
        }

        // Filter non-matching surrounding characters like brackets and quotes
        for surround_char in &SURROUND_CHARACTERS {
            // Store surrounding to find a match in the future
            if m.is_none() && surround_char.start() == &c {
                self.surround_states.push((*surround_char.end(), self.len));
                return ParserState::MaybeUrl;
            }

            // Truncate if there's no matching end for this start
            if surround_char.end() == &c {
                self.reset();
                return ParserState::NoUrl;
            }
        }

        match self.state {
            State::Default => self.advance_default(c),
            State::Path => self.advance_path(c),
            State::SchemeFirstSlash => self.advance_scheme_first_slash(c),
            State::SchemeSecondSlash => self.advance_scheme_second_slash(c),
            State::Scheme => {
                if let Some(length) = self.advance_scheme(c) {
                    self.reset();
                    return ParserState::Url(length);
                }
            },
        }

        match self.len {
            0 => ParserState::NoUrl,
            _ => ParserState::MaybeUrl,
        }
    }

    /// Reset the parser to its initial state.
    ///
    /// # Examples
    ///
    /// ```
    /// use rfind_url::{Parser, ParserState};
    ///
    /// let mut parser = Parser::new();
    ///
    /// // Feed some data into the parser
    /// for c in "ttps://example.org".chars().rev() {
    ///     assert_eq!(parser.advance(c), ParserState::MaybeUrl);
    /// }
    ///
    /// // Reset to initial state, ignoring the previously received characters
    /// parser.reset();
    ///
    /// // No URL detected, since the state has been reset
    /// assert_eq!(parser.advance('h'), ParserState::MaybeUrl);
    /// ```
    #[inline]
    pub fn reset(&mut self) {
        self.surround_states.clear();
        self.scheme_indices = [0; 8];
        self.state = State::Default;
        self.len = 0;
    }

    #[inline]
    fn advance_default(&mut self, c: char) {
        match c {
            '.' | ',' | ':'..=';' | '?' | '!' | '(' => self.reset(),
            _ => self.state = State::Path,
        }
    }

    #[inline]
    fn advance_path(&mut self, c: char) {
        if c == '/' {
            self.state = State::SchemeFirstSlash
        }
    }

    #[inline]
    fn advance_scheme_first_slash(&mut self, c: char) {
        if c == '/' {
            self.state = State::SchemeSecondSlash;
        } else {
            self.state = State::Path;
        }
    }

    #[inline]
    fn advance_scheme_second_slash(&mut self, c: char) {
        if c == ':' {
            self.state = State::Scheme;
        } else {
            self.state = State::Path;
        }
    }

    #[inline]
    fn advance_scheme(&mut self, c: char) -> Option<usize> {
        match c {
            'a'..='z' | 'A'..='Z' => {
                for (i, index) in self.scheme_indices.iter_mut().enumerate() {
                    let scheme_len = SCHEMES[i].len() as u8;

                    if *index >= scheme_len {
                        continue;
                    }

                    if SCHEMES[i].chars().rev().nth(*index as usize) != Some(c) {
                        *index = scheme_len + 1;
                    } else {
                        *index += 1;
                    }

                    // Returning early here is only possible because no scheme ends with another
                    // scheme. This is covered by the `no_scheme_conflicts` test.
                    if *index == scheme_len {
                        // Truncate the length to exclude all unmatched surroundings
                        self.len -= self.surround_states.last().map(|s| s.1).unwrap_or(0);

                        return Some(self.len);
                    }
                }
            },
            _ => self.reset(),
        }

        None
    }
}

#[inline]
fn is_illegal(c: char) -> bool {
    match c {
        '\u{00}'..='\u{1F}'
        | '\u{7F}'..='\u{9F}'
        | '<'
        | '>'
        | '"'
        | ' '
        | '{'..='}'
        | '\\'
        | '^'
        | '⟨'
        | '⟩'
        | '`' => true,
        _ => false,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SurroundCharacter {
    Bracket(char, char),
    Quote(char),
}

impl SurroundCharacter {
    #[inline]
    fn start(&self) -> &char {
        match self {
            SurroundCharacter::Bracket(_end, start) => &start,
            SurroundCharacter::Quote(quote) => &quote,
        }
    }

    #[inline]
    fn end(&self) -> &char {
        match self {
            SurroundCharacter::Bracket(end, _start) => &end,
            SurroundCharacter::Quote(quote) => &quote,
        }
    }
}

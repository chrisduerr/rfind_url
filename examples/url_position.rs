use rfind_url::{Parser, ParserState};

fn main() {
    let input = "before https://example.org after";

    let mut parser = Parser::new();
    for (i, c) in input.chars().rev().enumerate() {
        if let ParserState::Url(url_len) = parser.advance(c) {
            let url_start = input.len() - i - 1;
            let url_end = url_start + url_len as usize - 1;

            println!(
                "Input '{}' contains a URL starting at position '{}' going until position '{}'",
                input, url_start, url_end
            );

            assert_eq!(url_len, 19);
            assert_eq!(i, 24);

            break;
        }
    }
}

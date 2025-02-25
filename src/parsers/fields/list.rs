/*
 * Copyright Stalwart Labs Ltd. See the COPYING
 * file at the top-level directory of this distribution.
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

use std::borrow::Cow;

use crate::{decoders::encoded_word::decode_rfc2047, parsers::message::MessageStream, HeaderValue};

struct ListParser<'x> {
    token_start: usize,
    token_end: usize,
    is_token_start: bool,
    tokens: Vec<Cow<'x, str>>,
    list: Vec<Cow<'x, str>>,
}

fn add_token<'x>(parser: &mut ListParser<'x>, stream: &MessageStream<'x>, add_space: bool) {
    if parser.token_start > 0 {
        if !parser.tokens.is_empty() {
            parser.tokens.push(" ".into());
        }
        parser.tokens.push(String::from_utf8_lossy(
            &stream.data[parser.token_start - 1..parser.token_end],
        ));

        if add_space {
            parser.tokens.push(" ".into());
        }

        parser.token_start = 0;
        parser.is_token_start = true;
    }
}

fn add_tokens_to_list(parser: &mut ListParser) {
    if !parser.tokens.is_empty() {
        parser.list.push(if parser.tokens.len() == 1 {
            parser.tokens.pop().unwrap()
        } else {
            let value = parser.tokens.concat();
            parser.tokens.clear();
            value.into()
        });
    }
}

pub fn parse_comma_separared<'x>(stream: &mut MessageStream<'x>) -> HeaderValue<'x> {
    let mut parser = ListParser {
        token_start: 0,
        token_end: 0,
        is_token_start: true,
        tokens: Vec::new(),
        list: Vec::new(),
    };

    let mut iter = stream.data[stream.pos..].iter();

    while let Some(ch) = iter.next() {
        stream.pos += 1;
        match ch {
            b'\n' => {
                add_token(&mut parser, stream, false);

                match stream.data.get(stream.pos) {
                    Some(b' ' | b'\t') => {
                        if !parser.is_token_start {
                            parser.is_token_start = true;
                        }
                        iter.next();
                        stream.pos += 1;
                        continue;
                    }
                    _ => {
                        add_tokens_to_list(&mut parser);

                        return match parser.list.len() {
                            1 => HeaderValue::Text(parser.list.pop().unwrap()),
                            0 => HeaderValue::Empty,
                            _ => HeaderValue::TextList(parser.list),
                        };
                    }
                }
            }
            b' ' | b'\t' => {
                if !parser.is_token_start {
                    parser.is_token_start = true;
                }
                continue;
            }
            b'=' if parser.is_token_start => {
                if let (bytes_read, Some(token)) = decode_rfc2047(stream, stream.pos) {
                    add_token(&mut parser, stream, true);
                    parser.tokens.push(token.into());
                    stream.pos += bytes_read;
                    iter = stream.data[stream.pos..].iter();
                    continue;
                }
            }
            b',' => {
                add_token(&mut parser, stream, false);
                add_tokens_to_list(&mut parser);
                continue;
            }
            b'\r' => continue,
            _ => (),
        }

        if parser.is_token_start {
            parser.is_token_start = false;
        }

        if parser.token_start == 0 {
            parser.token_start = stream.pos;
        }

        parser.token_end = stream.pos;
    }

    HeaderValue::Empty
}

#[cfg(test)]
mod tests {
    use crate::parsers::fields::list::parse_comma_separared;
    use crate::parsers::message::MessageStream;
    use crate::HeaderValue;

    #[test]
    fn parse_comma_separated_text() {
        let inputs = [
            (" one item  \n", vec!["one item"]),
            ("simple, list\n", vec!["simple", "list"]),
            (
                "multi \r\n list, \r\n with, cr lf  \r\n",
                vec!["multi list", "with", "cr lf"],
            ),
            (
                "=?iso-8859-1?q?this is some text?=, in, a, list, \n",
                vec!["this is some text", "in", "a", "list"],
            ),
            (
                concat!(
                    " =?ISO-8859-1?B?SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=?=\n     ",
                    "=?ISO-8859-2?B?dSB1bmRlcnN0YW5kIHRoZSBleGFtcGxlLg==?=\n",
                    " , but, in a list, which, is, more, fun!\n"
                ),
                vec![
                    "If you can read this you understand the example.",
                    "but",
                    "in a list",
                    "which",
                    "is",
                    "more",
                    "fun!",
                ],
            ),
            (
                "=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=\n , listed\n",
                vec!["ab", "listed"],
            ),
            (
                "ハロー・ワールド, and also, ascii terms\n",
                vec!["ハロー・ワールド", "and also", "ascii terms"],
            ),
        ];

        for input in inputs {
            let str = input.0.to_string();

            match parse_comma_separared(&mut MessageStream::new(str.as_bytes())) {
                HeaderValue::TextList(ids) => {
                    assert_eq!(ids, input.1, "Failed to parse '{:?}'", input.0);
                }
                HeaderValue::Text(id) => {
                    assert!(input.1.len() == 1, "Failed to parse '{:?}'", input.0);
                    assert_eq!(id, input.1[0], "Failed to parse '{:?}'", input.0);
                }
                _ => panic!("Unexpected result"),
            }
        }
    }
}

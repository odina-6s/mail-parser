/*
 * Copyright Stalwart Labs, Minter Ltd. See the COPYING
 * file at the top-level directory of this distribution.
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

use std::borrow::Cow;

use crate::parsers::message_stream::MessageStream;

pub fn parse_raw<'x>(stream: &MessageStream<'x>) -> Option<Cow<'x, str>> {
    let mut token_start: usize = 0;
    let mut token_end: usize = 0;

    while let Some(ch) = stream.next() {
        match ch {
            b'\n' => match stream.peek() {
                Some(b' ' | b'\t') => {
                    stream.advance(1);
                    continue;
                }
                _ => {
                    return if token_start > 0 {
                        stream
                            .get_string(token_start - 1, token_end)
                            .unwrap()
                            .into()
                    } else {
                        None
                    };
                }
            },
            b' ' | b'\t' | b'\r' => continue,
            _ => (),
        }

        if token_start == 0 {
            token_start = stream.get_pos();
        }

        token_end = stream.get_pos();
    }

    None
}

pub fn parse_and_ignore(stream: &MessageStream) {
    while let Some(ch) = stream.next() {
        if ch == &b'\n' {
            match stream.peek() {
                Some(b' ' | b'\t') => {
                    stream.advance(1);
                    continue;
                }
                _ => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::{fields::raw::parse_raw, message_stream::MessageStream};

    #[test]
    fn parse_raw_text() {
        let inputs = [
            ("Saying Hello\nMessage-Id", "Saying Hello"),
            ("Re: Saying Hello\r\n \r\nFrom:", "Re: Saying Hello"),
            (
                concat!(
                    " from x.y.test\n      by example.net\n      via TCP\n",
                    "      with ESMTP\n      id ABC12345\n      ",
                    "for <mary@example.net>;  21 Nov 1997 10:05:43 -0600\n"
                ),
                concat!(
                    "from x.y.test\n      by example.net\n      via TCP\n",
                    "      with ESMTP\n      id ABC12345\n      ",
                    "for <mary@example.net>;  21 Nov 1997 10:05:43 -0600"
                ),
            ),
        ];

        for input in inputs {
            let str = input.0.to_string();
            assert_eq!(
                parse_raw(&MessageStream::new(str.as_bytes())).unwrap(),
                input.1,
                "Failed for '{:?}'",
                input.0
            );
        }
    }
}

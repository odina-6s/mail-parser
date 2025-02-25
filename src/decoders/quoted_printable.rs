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

use crate::parsers::message::MessageStream;

use super::DecodeResult;

#[derive(PartialEq, Debug)]
enum QuotedPrintableState {
    None,
    Eq,
    Hex1,
}

pub fn decode_quoted_printable<'x>(
    stream: &MessageStream<'x>,
    start_pos: usize,
    boundary: &[u8],
    is_word: bool,
) -> (usize, DecodeResult) {
    let mut success = boundary.is_empty();

    let mut bytes_read = 0;

    let mut buf = Vec::with_capacity(stream.data.len() - start_pos);

    let mut state = QuotedPrintableState::None;
    let mut match_count = 0;
    let mut hex1 = 0;
    let mut lf_pos = 0;

    debug_assert!(boundary.is_empty() || boundary.len() >= 2);

    for ch in stream.data[start_pos..].iter() {
        bytes_read += 1;

        // if success is false, a boundary was provided
        if !success {
            let mut is_boundary_end = true;

            if *ch == boundary[match_count] {
                match_count += 1;
                if match_count == boundary.len() {
                    let done = if is_word {
                        true
                    } else {
                        is_boundary_end =
                            crate::parsers::mime::is_boundary_end(stream, start_pos + bytes_read);
                        if is_boundary_end && bytes_read - boundary.len() == lf_pos {
                            buf.pop();
                        }
                        is_boundary_end
                    };

                    if done {
                        success = true;
                        break;
                    }
                } else {
                    continue;
                }
            }

            if match_count > 0 {
                if QuotedPrintableState::Eq != state {
                    for ch in boundary[..match_count].iter() {
                        buf.push(*ch);
                    }
                }
                state = QuotedPrintableState::None;

                // is_boundary_end is always true except in the rare event
                // that a boundary was found without a proper MIME ending (-- or \r\n)
                if is_boundary_end {
                    if *ch == boundary[0] {
                        // Char matched beginning of boundary, get next char.
                        match_count = 1;
                        continue;
                    } else {
                        // Reset match count and decode character
                        match_count = 0;
                    }
                } else {
                    // There was a full boundary match but without a proper MIME
                    // ending, reset match count and continue.
                    match_count = 0;
                    continue;
                }
            }
        }

        match ch {
            b'=' => {
                if let QuotedPrintableState::None = state {
                    state = QuotedPrintableState::Eq
                } else {
                    success = false;
                    break;
                }
            }
            b'\n' => {
                if is_word {
                    success = false;
                    break;
                } else if QuotedPrintableState::Eq == state {
                    state = QuotedPrintableState::None;
                } else {
                    buf.push(b'\n');
                    lf_pos = bytes_read;
                }
            }
            b'_' if is_word => {
                buf.push(b' ');
            }
            b'\r' => (),
            _ => match state {
                QuotedPrintableState::None => {
                    buf.push(*ch);
                }
                QuotedPrintableState::Eq => {
                    hex1 = HEX_MAP[*ch as usize];
                    if hex1 != -1 {
                        state = QuotedPrintableState::Hex1;
                    } else {
                        success = false;
                        break;
                    }
                }
                QuotedPrintableState::Hex1 => {
                    let hex2 = HEX_MAP[*ch as usize];

                    state = QuotedPrintableState::None;
                    if hex2 != -1 {
                        let ch = ((hex1 as u8) << 4) | hex2 as u8;

                        buf.push(ch);
                    } else {
                        success = false;
                        break;
                    }
                }
            },
        }
    }

    (
        if success { bytes_read } else { 0 },
        if !buf.is_empty() {
            buf.shrink_to_fit();
            DecodeResult::Owned(buf)
        } else {
            DecodeResult::Empty
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        decoders::{quoted_printable::decode_quoted_printable, DecodeResult},
        parsers::message::MessageStream,
    };

    #[test]
    fn decode_quoted_printable_string() {
        let inputs = [
            (
                concat!(
                    "J'interdis aux marchands de vanter trop leurs marchandises. ",
                    "Car ils se font=\nvite p=C3=A9dagogues et t'enseignent comme but ce ",
                    "qui n'est par essence qu=\n'un moyen, et te trompant ainsi sur la route ",
                    "=C3=A0 suivre les voil=C3=\n=A0 bient=C3=B4t qui te d=C3=A9gradent, car ",
                    "si leur musique est vulgaire il=\ns te fabriquent pour te la vendre une ",
                    "=C3=A2me vulgaire.\n=E2=80=94=E2=80=89Antoine de Saint-Exup=C3=A9ry, ",
                    "Citadelle (1948)"
                ),
                concat!(
                    "J'interdis aux marchands de vanter trop leurs marchandises. ",
                    "Car ils se fontvite pédagogues et t'enseignent comme but ce qui ",
                    "n'est par essence qu'un moyen, et te trompant ainsi sur la route ",
                    "à suivre les voilà bientôt qui te dégradent, car si leur musique ",
                    "est vulgaire ils te fabriquent pour te la vendre une âme vulgaire.\n",
                    "— Antoine de Saint-Exupéry, Citadelle (1948)"
                ),
                "",
                false,
            ),
            (
                "=E2=80=94=E2=80=89Antoine de Saint-Exup=C3=A9ry\n--boundary",
                "— Antoine de Saint-Exupéry",
                "--boundary",
                false,
            ),
            (
                "=E2=80=94=E2=80=89Antoine de Saint-Exup=C3=A9ry\n--\n--boundary",
                "— Antoine de Saint-Exupéry\n--",
                "--boundary",
                false,
            ),
            (
                "=E2=80=94=E2=80=89Antoine de Saint-Exup=C3=A9ry=\n--\n--boundary",
                "— Antoine de Saint-Exupéry--",
                "--boundary",
                false,
            ),
            (
                concat!(
                    "Die Hasen klagten einst uber ihre Lage; \"wir ",
                    "leben\", sprach ein=\r\n Redner, \"in steter Furcht vor Menschen",
                    " und Tieren, eine Beute der Hunde,=\r\n der\n"
                ),
                concat!(
                    "Die Hasen klagten einst uber ihre Lage; \"wir leben\", ",
                    "sprach ein Redner, \"in steter Furcht vor Menschen und ",
                    "Tieren, eine Beute der Hunde, der\n"
                ),
                "",
                false,
            ),
            ("this=20is=20some=20text?=", "this is some text", "?=", true),
            ("this is some text?=", "this is some text", "?=", true),
            ("Keith_Moore?=", "Keith Moore", "?=", true),
            ("=2=123?=", "", "?=", true),
            ("= 20?=", "", "?=", true),
            ("=====?=", "", "?=", true),
            ("=20=20=XX?=", "", "?=", true),
            ("=AX?=", "", "?=", true),
            ("=\n=\n==?=", "", "?=", true),
            ("=\r=1z?=", "", "?=", true),
            ("=|?=", "", "?=", true),
            ("????????=", "???????", "?=", true),
        ];

        for input in inputs {
            let str = input.0.to_string();
            let stream = MessageStream::new(str.as_bytes());

            let (bytes_read, result) =
                decode_quoted_printable(&stream, 0, input.2.as_bytes(), input.3);

            assert_eq!(
                bytes_read > 0,
                !input.1.is_empty(),
                "Failed for '{:?}'",
                input.0
            );

            if !input.1.is_empty() {
                let bytes = match result {
                    DecodeResult::Owned(v) => v,
                    _ => unreachable!(),
                };
                let result_str = std::str::from_utf8(&bytes).unwrap();

                /*println!(
                    "Decoded '{}'\n -> to ->\n'{}'\n{}",
                    input.0.escape_debug(),
                    result_str.escape_debug(),
                    "-".repeat(50)
                );*/

                assert_eq!(
                    input.1,
                    result_str,
                    "Failed for '{}'",
                    input.0.escape_debug()
                );
            }
        }

        // MIRI overflow tests
        for input in [b"\x0a\x0a\x00\x0b".to_vec(), b":B\x0a%".to_vec()] {
            let stream = MessageStream::new(&input[..]);
            decode_quoted_printable(&stream, 0, b"\n\n", true);
            let stream = MessageStream::new(&input[..]);
            decode_quoted_printable(&stream, 0, b"\n\n", false);
            let stream = MessageStream::new(&input[..]);
            decode_quoted_printable(&stream, 0, &[], true);
            let stream = MessageStream::new(&input[..]);
            decode_quoted_printable(&stream, 0, &[], false);
        }
    }
}

/*
 * Adapted from Daniel Lemire's source:
 * https://github.com/lemire/Code-used-on-Daniel-Lemire-s-blog/blob/master/2019/04/17/hexparse.cpp
 *
 */

pub static HEX_MAP: &[i8] = &[
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, -1, -1, -1, -1, -1, -1, -1, 10, 11, 12, 13, 14, 15, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10,
    11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
];

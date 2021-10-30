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

use super::{
    multi_byte::*,
    single_byte::*,
    utf::{decoder_utf16, decoder_utf16_be, decoder_utf16_le, decoder_utf7, decoder_utf8},
    DecoderFnc,
};

pub fn get_charset_decoder<'x>(charset: &[u8]) -> Option<DecoderFnc<'x>> {
    if (2..=45).contains(&charset.len()) {
        let mut l_charset = [0u8; 45];
        let mut hash: u32 = charset.len() as u32;

        for (pos, ch) in charset.iter().enumerate() {
            let ch = if (b'A'..=b'Z').contains(ch) {
                *ch + 32
            } else {
                *ch
            };
            unsafe {
                *l_charset.get_unchecked_mut(pos) = ch;
            }
            if let 0 | 3 | 6 | 7 | 8 | 9 = pos {
                hash += unsafe { *CH_HASH.get_unchecked(ch as usize) };
            }
            if pos == charset.len() - 1 {
                hash += unsafe { *CH_HASH.get_unchecked(ch as usize) };
            }
        }

        if (7..=764).contains(&hash) {
            let hash = (hash - 7) as usize;
            let &ch_map = unsafe { CH_MAP.get_unchecked(hash) };

            if l_charset[..charset.len()].eq(ch_map) {
                return Some(unsafe { *FNC_MAP.get_unchecked(hash) });
            }
        }
    }

    None
}

pub fn decoder_default(bytes: &[u8]) -> Cow<str> {
    String::from_utf8_lossy(bytes)
}

#[cfg(test)]
mod tests {
    use super::get_charset_decoder;

    #[test]
    fn get_decoder_charset() {
        let inputs = [
            "l8",
            "utf-8",
            "utf-7",
            "US-Ascii",
            "csgb18030",
            "iso-8859-1",
            "extended_unix_code_packed_format_for_japanese",
        ];

        for input in inputs {
            assert!(
                get_charset_decoder(input.as_bytes()).is_some(),
                "Failed for '{}'",
                input
            );
        }
    }
}

// Perfect hashing table for charset names
static CH_HASH: &[u32] = &[
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 0, 55, 765, 125, 5, 90, 155, 35, 15, 45, 140, 0, 30, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 15, 765,
    5, 0, 225, 15, 35, 0, 135, 115, 0, 5, 15, 5, 0, 20, 0, 30, 765, 0, 5, 10, 10, 765, 5, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765, 765,
    765,
];

static CH_MAP: &[&[u8]] = &[
    b"l8",
    b"",
    b"",
    b"",
    b"latin8",
    b"l1",
    b"",
    b"",
    b"utf-8",
    b"latin1",
    b"us",
    b"",
    b"",
    b"",
    b"koi8-r",
    b"l5",
    b"us-ascii",
    b"",
    b"",
    b"latin5",
    b"",
    b"ms_kanji",
    b"shift_jis",
    b"",
    b"koi8-u",
    b"",
    b"",
    b"big5",
    b"",
    b"ibm819",
    b"",
    b"",
    b"",
    b"",
    b"euc-kr",
    b"l4",
    b"",
    b"",
    b"",
    b"latin4",
    b"",
    b"866",
    b"",
    b"iso-ir-148",
    b"ibm866",
    b"l6",
    b"ecma-118",
    b"",
    b"iso-8859-8",
    b"latin6",
    b"",
    b"",
    b"",
    b"",
    b"utf-16",
    b"",
    b"",
    b"",
    b"iso-8859-1",
    b"iso-8859-11",
    b"",
    b"",
    b"",
    b"iso_8859-8",
    b"euc-jp",
    b"latin-9",
    b"",
    b"iso646-us",
    b"iso_8859-8:1988",
    b"iso-8859-15",
    b"",
    b"",
    b"",
    b"iso_8859-1",
    b"iso_8859-14:1998",
    b"",
    b"",
    b"",
    b"iso-8859-5",
    b"iso_8859-16:2001",
    b"",
    b"utf-16be",
    b"",
    b"iso_8859-5:1988",
    b"iso_8859-15",
    b"",
    b"utf-16le",
    b"",
    b"",
    b"iso-8859-14",
    b"l2",
    b"iso-ir-6",
    b"",
    b"iso_8859-5",
    b"latin2",
    b"",
    b"",
    b"",
    b"iso-ir-199",
    b"iso-8859-16",
    b"",
    b"",
    b"",
    b"iso_8859-4:1988",
    b"iso_8859-14",
    b"",
    b"",
    b"",
    b"iso-8859-9",
    b"",
    b"",
    b"",
    b"",
    b"iso-ir-144",
    b"iso_8859-16",
    b"",
    b"ecma-114",
    b"",
    b"iso-8859-4",
    b"hebrew",
    b"",
    b"850",
    b"",
    b"iso_8859-9",
    b"ibm850",
    b"windows-1258",
    b"l10",
    b"",
    b"iso_8859-9:1989",
    b"iso_646.irv:1991",
    b"windows-1251",
    b"asmo-708",
    b"",
    b"iso_8859-4",
    b"",
    b"",
    b"elot_928",
    b"",
    b"iso-8859-6",
    b"",
    b"windows-1255",
    b"",
    b"",
    b"iso-ir-101",
    b"",
    b"",
    b"gbk",
    b"",
    b"utf-7",
    b"",
    b"",
    b"",
    b"",
    b"iso_8859-6",
    b"",
    b"l3",
    b"",
    b"",
    b"",
    b"latin3",
    b"windows-1254",
    b"",
    b"",
    b"iso-ir-138",
    b"iso_8859-10:1992",
    b"",
    b"",
    b"",
    b"",
    b"greek8",
    b"windows-1256",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"iso-8859-10",
    b"",
    b"",
    b"",
    b"greek",
    b"",
    b"",
    b"",
    b"",
    b"iso-ir-126",
    b"",
    b"",
    b"",
    b"",
    b"iso-ir-109",
    b"",
    b"",
    b"",
    b"",
    b"ms936",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"windows-874",
    b"",
    b"",
    b"",
    b"",
    b"iso-8859-13",
    b"",
    b"",
    b"",
    b"iso_8859-1:1987",
    b"",
    b"windows-1252",
    b"",
    b"",
    b"extended_unix_code_packed_format_for_japanese",
    b"iso-2022-jp",
    b"",
    b"mac",
    b"",
    b"iso_8859-3:1988",
    b"",
    b"cskoi8r",
    b"",
    b"",
    b"iso-8859-2",
    b"arabic",
    b"csascii",
    b"",
    b"",
    b"",
    b"csutf8",
    b"cseuckr",
    b"",
    b"macintosh",
    b"csgbk",
    b"csbig5",
    b"",
    b"",
    b"",
    b"iso_8859-2",
    b"",
    b"cskoi8u",
    b"cswindows1258",
    b"",
    b"",
    b"",
    b"windows-1250",
    b"cswindows1251",
    b"",
    b"iso_8859-6:1987",
    b"",
    b"latin10",
    b"",
    b"ansi_x3.4-1968",
    b"cp819",
    b"windows-936",
    b"tis-620",
    b"cswindows1255",
    b"",
    b"iso-ir-110",
    b"",
    b"windows-1257",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"iso-ir-226",
    b"csisolatin1",
    b"cswindows874",
    b"",
    b"",
    b"",
    b"csisolatinhebrew",
    b"windows-1253",
    b"cswindows1254",
    b"",
    b"",
    b"csisolatin5",
    b"",
    b"",
    b"",
    b"csisolatingreek",
    b"",
    b"",
    b"cswindows1256",
    b"",
    b"",
    b"ibm367",
    b"",
    b"",
    b"",
    b"iso_8859-2:1987",
    b"csiso885915",
    b"",
    b"",
    b"ansi_x3.4-1986",
    b"iso-ir-157",
    b"csisolatin4",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cseucpkdfmtjapanese",
    b"cp866",
    b"csisolatin6",
    b"",
    b"",
    b"",
    b"",
    b"csiso885914",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"csutf16",
    b"",
    b"",
    b"iso-8859-7",
    b"csiso885916",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cswindows1252",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"iso_8859-7",
    b"",
    b"",
    b"",
    b"",
    b"iso_8859-7:1987",
    b"",
    b"",
    b"",
    b"csutf16be",
    b"",
    b"",
    b"",
    b"",
    b"csutf16le",
    b"iso-8859-3",
    b"csisolatin2",
    b"",
    b"csibm866",
    b"",
    b"cp850",
    b"",
    b"",
    b"",
    b"",
    b"csshiftjis",
    b"",
    b"",
    b"cswindows1250",
    b"",
    b"iso_8859-3",
    b"csutf7",
    b"",
    b"",
    b"",
    b"iso-ir-127",
    b"",
    b"",
    b"",
    b"",
    b"iso-ir-100",
    b"csmacintosh",
    b"gb18030",
    b"cswindows1257",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cswindows1253",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cp367",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cp936",
    b"csisolatin3",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"csiso885913",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"iso-celtic",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"csisolatinarabic",
    b"",
    b"csisolatincyrillic",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cstis620",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"csiso2022jp",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cspc850multilingual",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"cyrillic",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"",
    b"csgb18030",
];

#[allow(clippy::type_complexity)]
static FNC_MAP: &[for<'x> fn(&'x [u8]) -> Cow<'x, str>] = &[
    decoder_iso_8859_14,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_14,
    decoder_iso_8859_1,
    decoder_default,
    decoder_default,
    decoder_utf8,
    decoder_iso_8859_1,
    decoder_utf8,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_koi8_r,
    decoder_iso_8859_9,
    decoder_utf8,
    decoder_default,
    decoder_default,
    decoder_iso_8859_9,
    decoder_default,
    decoder_shift_jis,
    decoder_shift_jis,
    decoder_default,
    decoder_koi8_u,
    decoder_default,
    decoder_default,
    decoder_big5,
    decoder_default,
    decoder_iso_8859_1,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_euc_kr,
    decoder_iso_8859_4,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_4,
    decoder_default,
    decoder_ibm866,
    decoder_default,
    decoder_iso_8859_9,
    decoder_ibm866,
    decoder_iso_8859_10,
    decoder_iso_8859_7,
    decoder_default,
    decoder_iso_8859_8,
    decoder_iso_8859_10,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_utf16,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_1,
    decoder_tis_620,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_8,
    decoder_euc_jp,
    decoder_iso_8859_15,
    decoder_default,
    decoder_utf8,
    decoder_iso_8859_8,
    decoder_iso_8859_15,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_1,
    decoder_iso_8859_14,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_5,
    decoder_iso_8859_16,
    decoder_default,
    decoder_utf16_be,
    decoder_default,
    decoder_iso_8859_5,
    decoder_iso_8859_15,
    decoder_default,
    decoder_utf16_le,
    decoder_default,
    decoder_default,
    decoder_iso_8859_14,
    decoder_iso_8859_2,
    decoder_utf8,
    decoder_default,
    decoder_iso_8859_5,
    decoder_iso_8859_2,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_14,
    decoder_iso_8859_16,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_4,
    decoder_iso_8859_14,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_9,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_5,
    decoder_iso_8859_16,
    decoder_default,
    decoder_iso_8859_6,
    decoder_default,
    decoder_iso_8859_4,
    decoder_iso_8859_8,
    decoder_default,
    decoder_ibm_850,
    decoder_default,
    decoder_iso_8859_9,
    decoder_ibm_850,
    decoder_cp1258,
    decoder_iso_8859_16,
    decoder_default,
    decoder_iso_8859_9,
    decoder_utf8,
    decoder_cp1251,
    decoder_iso_8859_6,
    decoder_default,
    decoder_iso_8859_4,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_default,
    decoder_iso_8859_6,
    decoder_default,
    decoder_cp1255,
    decoder_default,
    decoder_default,
    decoder_iso_8859_2,
    decoder_default,
    decoder_default,
    decoder_gbk,
    decoder_default,
    decoder_utf7,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_6,
    decoder_default,
    decoder_iso_8859_3,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_3,
    decoder_cp1254,
    decoder_default,
    decoder_default,
    decoder_iso_8859_8,
    decoder_iso_8859_10,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_cp1256,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_10,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_3,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_gbk,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_windows874,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_13,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_1,
    decoder_default,
    decoder_cp1252,
    decoder_default,
    decoder_default,
    decoder_euc_jp,
    decoder_iso2022_jp,
    decoder_default,
    decoder_macintosh,
    decoder_default,
    decoder_iso_8859_3,
    decoder_default,
    decoder_koi8_r,
    decoder_default,
    decoder_default,
    decoder_iso_8859_2,
    decoder_iso_8859_6,
    decoder_utf8,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_utf8,
    decoder_euc_kr,
    decoder_default,
    decoder_macintosh,
    decoder_gbk,
    decoder_big5,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_2,
    decoder_default,
    decoder_koi8_u,
    decoder_cp1258,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_cp1250,
    decoder_cp1251,
    decoder_default,
    decoder_iso_8859_6,
    decoder_default,
    decoder_iso_8859_16,
    decoder_default,
    decoder_utf8,
    decoder_iso_8859_1,
    decoder_gbk,
    decoder_tis_620,
    decoder_cp1255,
    decoder_default,
    decoder_iso_8859_4,
    decoder_default,
    decoder_cp1257,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_16,
    decoder_iso_8859_1,
    decoder_windows874,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_8,
    decoder_cp1253,
    decoder_cp1254,
    decoder_default,
    decoder_default,
    decoder_iso_8859_9,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_default,
    decoder_default,
    decoder_cp1256,
    decoder_default,
    decoder_default,
    decoder_utf8,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_2,
    decoder_iso_8859_15,
    decoder_default,
    decoder_default,
    decoder_utf8,
    decoder_iso_8859_10,
    decoder_iso_8859_4,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_euc_jp,
    decoder_ibm866,
    decoder_iso_8859_10,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_14,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_utf16,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_iso_8859_16,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_cp1252,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_7,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_utf16_be,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_utf16_le,
    decoder_iso_8859_3,
    decoder_iso_8859_2,
    decoder_default,
    decoder_ibm866,
    decoder_default,
    decoder_ibm_850,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_shift_jis,
    decoder_default,
    decoder_default,
    decoder_cp1250,
    decoder_default,
    decoder_iso_8859_3,
    decoder_utf7,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_6,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_1,
    decoder_macintosh,
    decoder_gb18030,
    decoder_cp1257,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_cp1253,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_utf8,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_gbk,
    decoder_iso_8859_3,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_13,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_14,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_6,
    decoder_default,
    decoder_iso_8859_5,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_tis_620,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso2022_jp,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_ibm_850,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_iso_8859_5,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_default,
    decoder_gb18030,
];

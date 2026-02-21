/// Variable-length integer encoding that preserves lexicographic ordering.
///
/// # Unsigned encoding
///
/// Uses a unary-prefix scheme: the number of leading 1-bits across header
/// byte(s) determines how many extra data bytes follow. Smaller values use
/// fewer bytes; larger values grow up to 18 bytes for u128.
///
/// # Signed encoding
///
/// Bit 7 of the first byte is the sign bit (1 = non-negative, 0 = negative).
/// The remaining 7 bits begin the same unary-prefix scheme for the magnitude.
/// Negative values encode (|v| - 1) and then complement all bits (except
/// leaving the sign bit as 0), so more-negative values produce smaller bytes.

use crate::error::{Error, Result};

// ── Level tables ───────────────────────────────────────────────────────

/// For unsigned encoding (8-bit first header):
///   Levels 0..=7: single header byte, `n` leading 1-bits, 0-bit terminator,
///     data_bits = (7 - n) + 8*n = 7 + 7*n.
///   Levels 8..=15: first header = 0xFF, second header has m=level-8 leading
///     1-bits, 0-bit terminator, data_bits = (7 - m) + 8*level.
///   Level 16: both headers = 0xFF, data_bits = 16 * 8 = 128.
const UNSIGNED_DATA_BITS: [u32; 17] = [
    7, 14, 21, 28, 35, 42, 49, 56,         // levels 0..=7
    71, 78, 85, 92, 99, 106, 113, 120, 128, // levels 8..=16
];

const UNSIGNED_OFFSETS: [u128; 17] = compute_unsigned_offsets();

const fn compute_unsigned_offsets() -> [u128; 17] {
    let mut offsets = [0u128; 17];
    let mut i = 1;
    while i < 17 {
        let bits = UNSIGNED_DATA_BITS[i - 1];
        if bits >= 128 {
            offsets[i] = u128::MAX; // sentinel; level 16 is the last
        } else {
            offsets[i] = offsets[i - 1] + (1u128 << bits);
        }
        i += 1;
    }
    offsets
}

/// For signed encoding (7-bit first sub-header, sign bit occupies bit 7):
///   Levels 0..=6: single header byte, `n` leading 1-bits in bits 6..0,
///     data_bits = (6 - n) + 8*n = 6 + 7*n.
///   Levels 7..=14: sub-header exhausted (0x7F), second header byte has
///     m=level-7 leading 1-bits, data_bits = (7 - m) + 8*level.
///   Level 15: second header = 0xFF, third header byte 0xxxxxxx,
///     data_bits = 7 + 8*15 = 127.
const SIGNED_DATA_BITS: [u32; 16] = [
    6, 13, 20, 27, 34, 41, 48,       // levels 0..=6
    63, 70, 77, 84, 91, 98, 105, 112, // levels 7..=14
    127,                              // level 15
];

const SIGNED_OFFSETS: [u128; 16] = compute_signed_offsets();

const fn compute_signed_offsets() -> [u128; 16] {
    let mut offsets = [0u128; 16];
    let mut i = 1;
    while i < 16 {
        let bits = SIGNED_DATA_BITS[i - 1];
        if bits >= 128 {
            offsets[i] = u128::MAX;
        } else {
            offsets[i] = offsets[i - 1] + (1u128 << bits);
        }
        i += 1;
    }
    offsets
}

fn find_level(v: u128, offsets: &[u128]) -> usize {
    let last = offsets.len() - 1;
    for level in 0..last {
        if v < offsets[level + 1] {
            return level;
        }
    }
    last
}

// ── Unsigned encoding ──────────────────────────────────────────────────

/// Encode an unsigned integer into variable-length bytes.
pub fn encode_uint(v: u128, out: &mut Vec<u8>) {
    let level = find_level(v, &UNSIGNED_OFFSETS);
    let data = v - UNSIGNED_OFFSETS[level];

    if level <= 7 {
        // Single header byte: level leading 1-bits, then 0, then (7-level) data bits
        let hdr_data_bits = 7 - level;
        let prefix = leading_ones_byte(level);
        let hdr_data = extract_top_bits(data, level, hdr_data_bits);
        out.push(prefix | hdr_data as u8);
    } else if level <= 15 {
        out.push(0xFF);
        let m = level - 8;
        let hdr_data_bits = 7 - m;
        let prefix = leading_ones_byte(m);
        let hdr_data = extract_top_bits(data, level, hdr_data_bits);
        out.push(prefix | hdr_data as u8);
    } else {
        // Level 16: both headers = 0xFF, no data bits in header
        out.push(0xFF);
        out.push(0xFF);
    }

    // Write `level` extra data bytes in big-endian
    write_be_tail(data, level, out);
}

/// Decode a variable-length unsigned integer.
/// Returns (value, bytes_consumed).
pub fn decode_uint(input: &[u8]) -> Result<(u128, usize)> {
    if input.is_empty() {
        return Err(Error::Eof);
    }

    let first = input[0];
    let (level, header_data, header_len) = if first != 0xFF {
        let n = first.leading_ones() as usize;
        let hdr_data_bits = 7 - n;
        let hdr_data = (first & low_mask_u8(hdr_data_bits)) as u128;
        (n, hdr_data, 1)
    } else {
        if input.len() < 2 {
            return Err(Error::Eof);
        }
        let second = input[1];
        if second != 0xFF {
            let m = second.leading_ones() as usize;
            let hdr_data_bits = 7 - m;
            let hdr_data = (second & low_mask_u8(hdr_data_bits)) as u128;
            (8 + m, hdr_data, 2)
        } else {
            (16, 0u128, 2)
        }
    };

    let total = header_len + level;
    if input.len() < total {
        return Err(Error::Eof);
    }

    let data = assemble_be(header_data, &input[header_len..total]);
    Ok((data + UNSIGNED_OFFSETS[level], total))
}

// ── Signed encoding ────────────────────────────────────────────────────

/// Encode a signed integer into variable-length bytes.
pub fn encode_sint(v: i128, out: &mut Vec<u8>) {
    if v >= 0 {
        let start = out.len();
        encode_sint_magnitude(v as u128, out);
        out[start] |= 0x80; // set sign bit = 1 (non-negative)
    } else {
        let magnitude = (-(v + 1)) as u128;
        let start = out.len();
        encode_sint_magnitude(magnitude, out);
        out[start] |= 0x80; // temporarily set sign = 1 so complement makes it 0
        for b in &mut out[start..] {
            *b = !*b;
        }
    }
}

/// Decode a variable-length signed integer.
/// Returns (value, bytes_consumed).
pub fn decode_sint(input: &[u8]) -> Result<(i128, usize)> {
    if input.is_empty() {
        return Err(Error::Eof);
    }

    let positive = (input[0] & 0x80) != 0;

    if positive {
        let first_sub = input[0] & 0x7F;
        let (mag, consumed) = decode_sint_magnitude(first_sub, &input[1..])?;
        Ok((mag as i128, consumed))
    } else {
        // Determine length, complement, decode
        let first_complemented = !input[0];
        let first_sub = first_complemented & 0x7F;
        let total_len = sint_total_len(first_sub, &input[1..], true)?;
        if input.len() < total_len {
            return Err(Error::Eof);
        }
        let mut buf = Vec::with_capacity(total_len);
        for i in 0..total_len {
            buf.push(!input[i]);
        }
        let sub = buf[0] & 0x7F;
        let (mag, consumed) = decode_sint_magnitude(sub, &buf[1..])?;
        debug_assert_eq!(consumed, total_len);
        Ok((-(mag as i128) - 1, total_len))
    }
}

/// Encode magnitude using the 7-bit sub-header scheme.
/// Bit 7 of first byte is left as 0 (caller sets sign bit).
fn encode_sint_magnitude(v: u128, out: &mut Vec<u8>) {
    let level = find_level(v, &SIGNED_OFFSETS);
    let data = v - SIGNED_OFFSETS[level];

    if level <= 6 {
        let hdr_data_bits = 6 - level;
        let prefix = leading_ones_7bit(level);
        let hdr_data = extract_top_bits(data, level, hdr_data_bits);
        out.push(prefix | hdr_data as u8);
    } else if level <= 14 {
        // First byte: sub-header = 0x7F (bit 7 = 0, bits 6..0 = all 1s)
        out.push(0x7F);
        let m = level - 7;
        let hdr_data_bits = 7 - m;
        let prefix = leading_ones_byte(m);
        let hdr_data = extract_top_bits(data, level, hdr_data_bits);
        out.push(prefix | hdr_data as u8);
    } else {
        // Level 15: first = 0x7F, second = 0xFF, third has 7 data bits
        out.push(0x7F);
        out.push(0xFF);
        let hdr_data = extract_top_bits(data, 15, 7);
        out.push(hdr_data as u8);
    }

    write_be_tail(data, level, out);
}

/// Decode magnitude from 7-bit sub-header and remaining bytes.
fn decode_sint_magnitude(sub: u8, rest: &[u8]) -> Result<(u128, usize)> {
    let (level, header_data, extra_header_bytes) = if sub != 0x7F {
        let n = leading_ones_in_7bits(sub);
        let hdr_data_bits = 6 - n;
        let hdr_data = (sub & low_mask_u8(hdr_data_bits)) as u128;
        (n, hdr_data, 0usize)
    } else {
        if rest.is_empty() {
            return Err(Error::Eof);
        }
        let second = rest[0];
        if second != 0xFF {
            let m = second.leading_ones() as usize;
            let hdr_data_bits = 7 - m;
            let hdr_data = (second & low_mask_u8(hdr_data_bits)) as u128;
            (7 + m, hdr_data, 1)
        } else {
            if rest.len() < 2 {
                return Err(Error::Eof);
            }
            let third = rest[1];
            let hdr_data = (third & 0x7F) as u128;
            (15, hdr_data, 2)
        }
    };

    let data_start = extra_header_bytes;
    let data_end = data_start + level;
    if rest.len() < data_end {
        return Err(Error::Eof);
    }

    let data = assemble_be(header_data, &rest[data_start..data_end]);
    let total_consumed = 1 + data_end; // 1 for first byte + rest consumed
    Ok((data + SIGNED_OFFSETS[level], total_consumed))
}

/// Determine total encoded byte length of a signed value.
fn sint_total_len(sub: u8, rest: &[u8], complemented: bool) -> Result<usize> {
    if sub != 0x7F {
        let n = leading_ones_in_7bits(sub);
        Ok(1 + n)
    } else {
        if rest.is_empty() {
            return Err(Error::Eof);
        }
        let second = if complemented { !rest[0] } else { rest[0] };
        if second != 0xFF {
            let m = second.leading_ones() as usize;
            let level = 7 + m;
            Ok(2 + level)
        } else {
            Ok(3 + 15) // level 15: 3 header + 15 data
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Count leading 1-bits in a 7-bit field (stored in bits 6..0 of a u8).
fn leading_ones_in_7bits(v: u8) -> usize {
    (v << 1).leading_ones() as usize
}

/// A byte with `n` leading 1-bits followed by 0s.
fn leading_ones_byte(n: usize) -> u8 {
    if n == 0 { 0 } else if n >= 8 { 0xFF } else { !0u8 << (8 - n) }
}

/// Byte with bits 6..0 having `n` leading 1-bits (bit 7 always 0).
fn leading_ones_7bit(n: usize) -> u8 {
    if n == 0 { 0 } else { leading_ones_byte(n) >> 1 }
}

/// Low-bit mask: low_mask_u8(3) = 0b0000_0111.
fn low_mask_u8(bits: usize) -> u8 {
    if bits == 0 { 0 } else if bits >= 8 { 0xFF } else { (1u8 << bits) - 1 }
}

/// Extract the top `want` data bits from a value that has `extra_bytes*8` bits
/// in the tail. The header data bits are above the tail, so we shift right.
fn extract_top_bits(data: u128, extra_bytes: usize, want: usize) -> u128 {
    if want == 0 {
        return 0;
    }
    let shift = extra_bytes * 8;
    if shift >= 128 {
        return 0;
    }
    (data >> shift) & low_mask_128(want)
}

fn low_mask_128(bits: usize) -> u128 {
    if bits == 0 { 0 } else if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 }
}

/// Write the bottom `n` bytes of `data` in big-endian.
fn write_be_tail(data: u128, n: usize, out: &mut Vec<u8>) {
    for i in (0..n).rev() {
        let shift = i * 8;
        out.push(if shift >= 128 { 0 } else { (data >> shift) as u8 });
    }
}

/// Combine header data bits with subsequent big-endian bytes.
fn assemble_be(prefix: u128, bytes: &[u8]) -> u128 {
    let mut v = prefix;
    for &b in bytes {
        v = (v << 8) | b as u128;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leading_ones_in_7bits() {
        assert_eq!(leading_ones_in_7bits(0b0000_0000), 0);
        assert_eq!(leading_ones_in_7bits(0b0100_0000), 1);
        assert_eq!(leading_ones_in_7bits(0b0110_0000), 2);
        assert_eq!(leading_ones_in_7bits(0b0111_0000), 3);
        assert_eq!(leading_ones_in_7bits(0b0111_1111), 7);
    }

    #[test]
    fn test_uint_roundtrip_small() {
        for v in 0u128..=300 {
            let mut buf = Vec::new();
            encode_uint(v, &mut buf);
            let (decoded, consumed) = decode_uint(&buf).unwrap();
            assert_eq!(v, decoded, "roundtrip failed for {v}");
            assert_eq!(consumed, buf.len(), "consumed mismatch for {v}");
        }
    }

    #[test]
    fn test_uint_order_preservation() {
        let values: Vec<u128> = vec![
            0, 1, 63, 64, 126, 127, 128, 255, 256,
            16511, 16512, 65535, 65536,
            1 << 20, 1 << 28, 1 << 35, 1 << 42, 1 << 49,
            (1u128 << 56) - 1, 1 << 56,
            1 << 63, u64::MAX as u128,
            (u64::MAX as u128) + 1,
            1u128 << 70, 1u128 << 100, 1u128 << 120, 1u128 << 127,
            u128::MAX - 1, u128::MAX,
        ];
        let encoded: Vec<Vec<u8>> = values.iter().map(|&v| {
            let mut buf = Vec::new();
            encode_uint(v, &mut buf);
            buf
        }).collect();
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert!(
                    encoded[i] < encoded[j],
                    "{} (enc {:?}) should be < {} (enc {:?})",
                    values[i], encoded[i], values[j], encoded[j]
                );
            }
        }
    }

    #[test]
    fn test_uint_roundtrip_boundaries() {
        for level in 0..17usize {
            let offset = UNSIGNED_OFFSETS[level];
            let mut buf = Vec::new();
            encode_uint(offset, &mut buf);
            let (decoded, consumed) = decode_uint(&buf).unwrap();
            assert_eq!(offset, decoded, "boundary start failed at level {level}");
            assert_eq!(consumed, buf.len());

            if offset < u128::MAX {
                buf.clear();
                let v = offset + 1;
                encode_uint(v, &mut buf);
                let (decoded, consumed) = decode_uint(&buf).unwrap();
                assert_eq!(v, decoded, "boundary start+1 failed at level {level}");
                assert_eq!(consumed, buf.len());
            }

            if level < 16 {
                buf.clear();
                let v = UNSIGNED_OFFSETS[level + 1] - 1;
                encode_uint(v, &mut buf);
                let (decoded, consumed) = decode_uint(&buf).unwrap();
                assert_eq!(v, decoded, "boundary end failed at level {level}");
                assert_eq!(consumed, buf.len());
            }
        }
        let mut buf = Vec::new();
        encode_uint(u128::MAX, &mut buf);
        let (decoded, consumed) = decode_uint(&buf).unwrap();
        assert_eq!(u128::MAX, decoded);
        assert_eq!(consumed, buf.len());
    }

    #[test]
    fn test_sint_roundtrip_small() {
        for v in -300i128..=300 {
            let mut buf = Vec::new();
            encode_sint(v, &mut buf);
            let (decoded, consumed) = decode_sint(&buf).unwrap();
            assert_eq!(v, decoded, "roundtrip failed for {v}");
            assert_eq!(consumed, buf.len(), "consumed mismatch for {v}");
        }
    }

    #[test]
    fn test_sint_order_preservation() {
        let values: Vec<i128> = vec![
            i128::MIN,
            i128::MIN + 1,
            i64::MIN as i128,
            -1_000_000,
            -1000,
            -128,
            -127,
            -64,
            -1,
            0,
            1,
            63,
            127,
            128,
            1000,
            1_000_000,
            i64::MAX as i128,
            i128::MAX - 1,
            i128::MAX,
        ];
        let encoded: Vec<Vec<u8>> = values.iter().map(|&v| {
            let mut buf = Vec::new();
            encode_sint(v, &mut buf);
            buf
        }).collect();
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert!(
                    encoded[i] < encoded[j],
                    "{} (enc {:?}) should be < {} (enc {:?})",
                    values[i], encoded[i], values[j], encoded[j]
                );
            }
        }
    }

    #[test]
    fn test_sint_roundtrip_extremes() {
        for &v in &[i128::MIN, i128::MIN + 1, -1, 0, 1, i128::MAX - 1, i128::MAX] {
            let mut buf = Vec::new();
            encode_sint(v, &mut buf);
            let (decoded, consumed) = decode_sint(&buf).unwrap();
            assert_eq!(v, decoded, "roundtrip failed for {v}");
            assert_eq!(consumed, buf.len(), "consumed mismatch for {v}");
        }
    }

    #[test]
    fn test_uint_compactness() {
        let mut buf = Vec::new();

        encode_uint(0, &mut buf);
        assert_eq!(buf.len(), 1);

        buf.clear();
        encode_uint(127, &mut buf);
        assert_eq!(buf.len(), 1);

        buf.clear();
        encode_uint(128, &mut buf);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_sint_compactness() {
        let mut buf = Vec::new();

        encode_sint(0, &mut buf);
        assert_eq!(buf.len(), 1);

        buf.clear();
        encode_sint(-1, &mut buf);
        assert_eq!(buf.len(), 1);

        buf.clear();
        encode_sint(63, &mut buf);
        assert_eq!(buf.len(), 1);

        buf.clear();
        encode_sint(-64, &mut buf);
        assert_eq!(buf.len(), 1);

        buf.clear();
        encode_sint(64, &mut buf);
        assert_eq!(buf.len(), 2);
    }
}

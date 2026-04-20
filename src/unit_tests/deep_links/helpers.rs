use crate::types::resource::Stream;

/// Parse a `stremio:///player/{encoded_stream}[/suffix]` URL and assert that
/// its encoded-stream segment round-trips back to `expected`, and that any
/// suffix after it matches `expected_suffix`.
///
/// This is the durable alternative to byte-for-byte URL comparison. The
/// encoded-stream segment is produced by `Stream::encode`, which zlib-compresses
/// the JSON payload before base64 + percent-encoding; different versions of
/// `miniz_oxide` / `flate2` emit the same decompressed bytes through different
/// compressed byte sequences, so literal string comparisons break on every
/// compression-backend bump without any semantic regression. Checking the
/// decoded `Stream` instead keeps the public contract (URL grammar + payload
/// shape) under test while staying stable across upstream dep updates.
pub(crate) fn assert_player_url(actual: &str, expected: &Stream, expected_suffix: &str) {
    const PREFIX: &str = "stremio:///player/";
    let rest = actual
        .strip_prefix(PREFIX)
        .unwrap_or_else(|| panic!("expected `{actual}` to start with `{PREFIX}`"));
    let (encoded_stream, suffix) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let base64_str = percent_encoding::percent_decode_str(encoded_stream)
        .decode_utf8()
        .expect("percent-decoded stream segment is valid UTF-8");
    let decoded = Stream::decode(&base64_str)
        .unwrap_or_else(|e| panic!("failed to decode stream segment `{encoded_stream}`: {e}"));
    assert_eq!(
        &decoded, expected,
        "decoded Stream did not match the expected value"
    );
    assert_eq!(
        suffix, expected_suffix,
        "player URL suffix after encoded stream did not match"
    );
}

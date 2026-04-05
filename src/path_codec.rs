use anyhow::Context;
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};

const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

pub fn encode_path_segment_for_display(value: &str) -> String {
    utf8_percent_encode(value, PATH_SEGMENT_ENCODE_SET).to_string()
}

pub fn decode_path_segment_input(value: &str) -> anyhow::Result<String> {
    percent_decode_str(value)
        .decode_utf8()
        .map(|s| s.into_owned())
        .with_context(|| format!("failed to decode path segment: {value}"))
}

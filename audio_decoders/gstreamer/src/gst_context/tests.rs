use super::*;

const OGG_FILE: &'static [u8] = include_bytes!("test.ogg");
const DECODED_OGG_FILE: &'static str = include_str!("decoded_test.json");

#[test]
fn test_decode_ogg() {
    gst::init().unwrap();

    let ctx = GstContext::new(EncodedAudioInfo {
        format: AudioFormat::Ogg,
        codec: AudioCodec::Opus,
        sample_rate: 48000,
    });

    let encoded_audio = EncodedAudioBuffer(Vec::from(OGG_FILE));
    ctx.push(encoded_audio);
    ctx.push_eos();

    let mut decoded_audio = vec![];
    while !ctx.is_eos() {
        if ctx.is_playing_failed() {
            break;
        }

        let Some(audio) = ctx.pull() else { continue };
        decoded_audio.extend_from_slice(audio.as_slice());
    }

    let decoded_audio_json = serde_json::to_string(&decoded_audio).unwrap();
    assert_eq!(decoded_audio_json, DECODED_OGG_FILE);
}

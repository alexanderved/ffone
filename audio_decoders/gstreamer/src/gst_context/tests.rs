use super::*;

const OPUS_DATA: &'static str = include_str!("test.opus.data");
const RAW_DATA: &'static str = include_str!("test.raw.data");

#[test]
fn test_decode_opus() {
    gst::init().unwrap();

    let header = EncodedAudioHeader {
        codec: AudioCodec::Opus,
        sample_rate: 48000,
    };

    let ctx = GstContext::new(header);

    let opus_buffers: Vec<Vec<u8>> = serde_json::from_str(OPUS_DATA).unwrap();
    for data in opus_buffers {
        let encoded_audio = EncodedAudioBuffer {
            header,
            start_ts: Some(ClockTime::from_secs(10)),
            data,
        };
        ctx.push(encoded_audio);
    }
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
    assert_eq!(decoded_audio_json, RAW_DATA);
}

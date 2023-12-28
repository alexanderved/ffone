use crate::{
    audio_system::audio::{AudioCodec, EncodedAudioHeader},
    util::ClockTime,
};

use super::*;

#[test]
fn test_demux() {
    const TS_IN_NANOS: u64 = 100000000;

    let (send, _) = unidirectional_queue();
    let mut demuxer = AudioDemuxer::new(send);

    let muxed_buf = {
        let mut data = vec![42; 5 + 8 + 16];

        data[0] = AudioCodec::Opus as u8;
        data[1..5].copy_from_slice(&48000u32.to_be_bytes());
        data[5..5 + 8].copy_from_slice(&TS_IN_NANOS.to_be_bytes());

        MuxedAudioBuffer(data)
    };
    demuxer.push(muxed_buf);

    let encoded_buf = demuxer.pull().unwrap();
    let expected_encoded_buf = EncodedAudioBuffer {
        header: EncodedAudioHeader {
            codec: AudioCodec::Opus,
            sample_rate: 48000,
        },
        start_ts: Some(ClockTime::from_nanos(TS_IN_NANOS)),
        data: vec![42; 16],
    };

    assert_eq!(encoded_buf, expected_encoded_buf);
}

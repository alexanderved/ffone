use core::{
    audio_system::{
        audio::*,
        element::{AudioSink, AudioSource},
        pipeline::{
            demuxer::AudioDemuxer, resizer::AudioResizer, sync::Synchronizer,
            virtual_microphone::VirtualMicrophone,
        },
    },
    mueue::unidirectional_queue,
    serde::de,
    util::{Runnable, SystemClock},
};
use gstreamer::GstDecoder;
use pulseaudio::PAVirtualMicrophone;
use std::{f64::consts::PI, sync::Arc};

const OPUS_DATA: &'static str = include_str!("test.opus.data");
const RAW_DATA: &'static str = include_str!("test.raw.data");

fn main() {
    //gst::init().unwrap();

    let (send, _) = unidirectional_queue();
    let sys_clock = Arc::new(SystemClock::new());

    let mut dec = GstDecoder::new(send.clone());
    let mut sync = Synchronizer::new(send.clone(), sys_clock);
    let mut resizer = AudioResizer::new(send.clone());
    let mut virtual_mic = PAVirtualMicrophone::new(send).unwrap();

    dec.chain(&mut sync);
    sync.chain(&mut resizer);
    resizer.chain(&mut virtual_mic);

    sync.set_virtual_microphone_statistics(virtual_mic.provide_statistics());

    let input = sync.create_input();

    /* let header = EncodedAudioHeader {
        codec: AudioCodec::Opus,
        sample_rate: 48000,
    };

    let opus_buffers: Vec<Vec<u8>> = serde_json::from_str(OPUS_DATA).unwrap();
    for data in opus_buffers {
        let encoded_audio = EncodedAudioBuffer {
            header,
            start_ts: None,
            data,
        };
        let _ = input.send(encoded_audio);
    } */

    let mut accum = 0.0;
    let freq = 440.0f64;
    let sample_rate = 8000;

    loop {
        let mut data: Vec<u8> = vec![];
        for _ in 0..1000 {
            accum += freq * 2.0 * PI / (sample_rate as f64);
            if accum >= PI * 2.0 {
                accum -= PI * 2.0;
            }

            let wave = (accum.sin() * i16::MAX as f64) as i16;
            let bytes = wave.to_le_bytes();

            data.extend(bytes);
        }

        let raw = RawAudioBuffer::new(data, RawAudioFormat::S16LE, sample_rate);
        let ts_buf = TimestampedRawAudioBuffer::new(raw, None);

        let _ = input.send(ts_buf);
        // freq += 0.5;

        //let _ = dec.update(None);
        let _ = sync.update();
        let _ = resizer.update();
        let _ = virtual_mic.update();
    }
    
    /*
    let ctx = GstContext::new(header);

    let opus_buffers: Vec<Vec<u8>> = serde_json::from_str(OPUS_DATA).unwrap();
    for data in opus_buffers {
        let encoded_audio = EncodedAudioBuffer {
            header,
            start_ts: ClockTime::from_secs(10),
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
    assert_eq!(decoded_audio_json, RAW_DATA); */
}

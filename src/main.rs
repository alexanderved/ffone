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
    util::{ClockTime, Runnable, SystemClock},
};
use gstreamer::{GstDecoder, GstContext};
use pulseaudio::PAVirtualMicrophone;
use std::{f64::consts::PI, sync::Arc};

use gst::{
    prelude::{Cast, GstBinExtManual},
    traits::ElementExt,
};

const OPUS_DATA: &'static str = include_str!("test.opus.data");
const RAW_DATA: &'static str = include_str!("test.raw.data");

fn main() {
    gst::init().unwrap();

    let (send, _) = unidirectional_queue();
    let sys_clock = Arc::new(SystemClock::new());

    let mut dec = GstDecoder::new(send.clone());
    let mut sync = Synchronizer::new(send.clone(), sys_clock);
    let mut resizer = AudioResizer::new(send.clone());
    let mut virtual_mic = PAVirtualMicrophone::new(send).unwrap();

    dec.chain(&mut sync);
    sync.chain(&mut resizer);
    resizer.chain(&mut virtual_mic);

    dec.on_start();
    sync.on_start();
    resizer.on_start();
    virtual_mic.on_start();

    sync.set_virtual_microphone_clock(virtual_mic.provide_clock());

    let input = dec.create_input();

    let header = EncodedAudioHeader {
        codec: AudioCodec::Opus,
        sample_rate: 48000,
    };



    let pipeline = gst::Pipeline::new(Some("gst_audio_decoder_pipeline"));

    let caps = gst::Caps::builder("audio/x-raw")
        .field("rate", 48000)
        .field("channels", 1)
        .field("format", "S16LE")
        .field("layout", "interleaved")
        .build();
    let src = gst_app::AppSrc::builder()
        .name("sr")
        .caps(&caps)
        .stream_type(gst_app::AppStreamType::Stream)
        .build();

    let encoder = gst::ElementFactory::make("opusenc")
        .name("encoder")
        .build()
        .unwrap();
    let sink = gst_app::AppSink::builder().name("s").build();
    //sink.set_sync(false);

    

    pipeline
        .add_many(&[src.upcast_ref(), &encoder, sink.upcast_ref()])
        .unwrap();
    gst::Element::link_many(&[src.upcast_ref(), &encoder, sink.upcast_ref()]).unwrap();

    pipeline.set_state(gst::State::Playing).unwrap();

    let mut accum = 0.0;
    let freq = 440.0f64;
    let sample_rate = 48000;
    for _ in 0..1920 * 4 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..100 {
            accum += freq * 2.0 * PI / (sample_rate as f64);
            if accum >= PI * 2.0 {
                accum -= PI * 2.0;
            }

            let wave = (accum.sin() * i16::MAX as f64) as i16;
            let bytes = wave.to_le_bytes();

            //println!("{}", accum.sin());

            data.extend(bytes);
        }

        //dbg!(pipeline.state(Some(gst::ClockTime::from_mseconds(10))));
        
        src.push_buffer(gst::Buffer::from_slice(data)).unwrap();
    }

    src.end_of_stream().unwrap();
    
    while !sink.is_eos() {
        let Some(sample) = sink
            .try_pull_sample(Some(gst::ClockTime::from_mseconds(1)))
        else {
            continue;
        };

        let buf = sample.buffer().unwrap();

        let encoded_audio = EncodedAudioBuffer {
            header,
            start_ts: None,
            data: buf.map_readable().unwrap().as_slice().to_owned(),
        };
        let _ = input.send(encoded_audio);
    }



    /* let opus_buffers: Vec<Vec<u8>> = serde_json::from_str(OPUS_DATA).unwrap();
    for data in opus_buffers {
        let encoded_audio = EncodedAudioBuffer {
            header,
            start_ts: None,
            data,
        };
        let _ = input.send(encoded_audio);
    } */

    let _ = dec.update();
    dec.drain();

    /* let mut accum = 0.0;
    let freq = 440.0f64;
    let sample_rate = 48000;

    for _ in 0..1920 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..100 {
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
    }

    for _ in 0..1920 {
        let raw = RawAudioBuffer::new(vec![0; 200], RawAudioFormat::S16LE, sample_rate);
        let ts_buf = TimestampedRawAudioBuffer::new(raw, None);

        let _ = input.send(ts_buf);
    }

    for _ in 0..1920 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..100 {
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
    }

    for _ in 0..1920 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..100 {
            accum += freq * 2.0 * 2.0 * PI / (sample_rate as f64);
            if accum >= PI * 2.0 {
                accum -= PI * 2.0;
            }

            let wave = (accum.sin() * i16::MAX as f64) as i16;
            let bytes = wave.to_be_bytes();

            data.extend(bytes);
        }

        let raw = RawAudioBuffer::new(data, RawAudioFormat::S16BE, sample_rate);
        let ts_buf = TimestampedRawAudioBuffer::new(raw, None);

        let _ = input.send(ts_buf);
    } */

    for _ in 0..50000000 {
        /* let mut data: Vec<u8> = vec![];
        for _ in 0..100 {
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

        let _ = input.send(ts_buf); */
        // freq += 0.5;

        // let _ = dec.update();
        let _ = sync.update();
        let _ = resizer.update();
        let _ = virtual_mic.update();
    }

    dec.on_stop();
    sync.on_stop();
    resizer.on_stop();
    virtual_mic.on_stop();
    

    

    /* let input = dec.create_input();
    let output = dec.create_output();

    let opus_buffers: Vec<Vec<u8>> = serde_json::from_str(OPUS_DATA).unwrap();
    for data in opus_buffers {
        let encoded_audio = EncodedAudioBuffer {
            header,
            start_ts: Some(ClockTime::from_secs(10)),
            data,
        };
        let _ = input.send(encoded_audio);
    }

    let _ = dec.update();
    dec.drain();

    let mut decoded_audio = vec![];
    for audio in output.iter() {
        decoded_audio.extend_from_slice(audio.as_slice());
    }

    let decoded_audio_json = serde_json::to_string(&decoded_audio).unwrap();
    assert_eq!(decoded_audio_json, RAW_DATA); */
}

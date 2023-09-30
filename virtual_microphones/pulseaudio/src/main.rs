use core::audio_system::audio::RawAudioBuffer;
use core::audio_system::audio::RawAudioFormat;
use core::audio_system::element::AudioSink;
use core::mueue::unidirectional_queue;
use core::util::Runnable;
use std::f64::consts::PI;

use ffone_pa_virtual_microphone::PAVirtualMicrophone;

fn main() {
    let (send, _) = unidirectional_queue();
    let mut mic = PAVirtualMicrophone::new(send).unwrap();
    let in_send = mic.create_input();
    
    let mut accum: f64 = 0.0;

    for _ in 0..8 * 3 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..1000 {
            accum += 440.0f64 * 2.0 * PI / 8000.0;
            if accum >= PI * 2.0 {
                accum -= PI * 2.0;
            }

            let wave = ((accum.sin() + 1.0) / 2.0 * 255.0) as u8;
            data.push(wave);
        }

        let _ = in_send.send(RawAudioBuffer::new(data, RawAudioFormat::U8, 8000));
    }

    for _ in 0..8 * 3 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..1000 {
            accum += 440.0f64 * 2.0 * PI / 8000.0;
            if accum >= PI * 2.0 {
                accum -= PI * 2.0;
            }

            let wave = (accum.sin() * i16::MAX as f64) as i16;
            let bytes = wave.to_le_bytes();

            data.extend(bytes);
        }

        let _ = in_send.send(RawAudioBuffer::new(data, RawAudioFormat::S16LE, 8000));
    }

    for _ in 0..44 * 3 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..1000 {
            accum += 440.0f64 * 2.0 * PI / 44100.0;
            if accum >= PI * 2.0 {
                accum -= PI * 2.0;
            }

            let wave = ((accum.sin() + 1.0) / 2.0 * 255.0) as u8;
            data.push(wave);
        }

        let _ = in_send.send(RawAudioBuffer::new(data, RawAudioFormat::U8, 44100));
    }

    for _ in 0..48 * 3 {
        let mut data: Vec<u8> = vec![];
        for _ in 0..1000 {
            accum += 440.0f64 * 2.0 * PI / 48000.0;
            if accum >= PI * 2.0 {
                accum -= PI * 2.0;
            }

            let wave = ((accum.sin() + 1.0) / 2.0 * 255.0) as u8;
            data.push(wave);
        }

        let _ = in_send.send(RawAudioBuffer::new(data, RawAudioFormat::U8, 48000));
    }

    loop {
        let _ = mic.update(None);

        if !mic.queue.has_bytes() {
            break;
        }
    }
}

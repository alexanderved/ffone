#[cfg(test)]
mod tests;

use core::audio_system::audio::{
    AudioCodec, AudioFormat, EncodedAudioBuffer, EncodedAudioInfo, RawAudioBuffer, RawAudioFormat,
    Timestamp, TimestampedRawAudioBuffer,
};
use std::time::Duration;

use gst::{
    prelude::{Cast, GstBinExtManual},
    traits::{ElementExt, PadExt},
};
use gstreamer as gst;
use gstreamer_app as gst_app;

#[allow(dead_code)]
pub(super) struct GstContext {
    pipeline: gst::Pipeline,

    src: gst_app::AppSrc,
    demuxer: gst::Element,
    parser: gst::Element,
    decoder: gst::Element,
    sink: gst_app::AppSink,
}

impl GstContext {
    pub(super) fn new(info: EncodedAudioInfo) -> Self {
        let pipeline = gst::Pipeline::new(Some("gst_audio_decoder_pipeline"));

        let caps = gst::Caps::builder(mime_from_format(info.format))
            .field("rate", info.sample_rate)
            .field("channels", 1)
            .build();
        let src = gst_app::AppSrc::builder()
            .name("src")
            .caps(&caps)
            .stream_type(gst_app::AppStreamType::Stream)
            .build();

        let demuxer = gst::ElementFactory::make(demuxer_name_from_format(info.format))
            .name("demuxer")
            .build()
            .unwrap();
        let parser = gst::ElementFactory::make(parser_name_from_codec(info.codec))
            .name("parser")
            .build()
            .unwrap();
        demuxer.connect_pad_added({
            let parser = parser.clone();
            move |_, pad| {
                let Some(sink_pad) = parser.static_pad("sink") else { return };
                let Some(caps) = pad.caps() else { return };

                for structure in caps.iter() {
                    let pad_type = structure.name();

                    if pad_type.starts_with(mime_from_codec(info.codec)) {
                        pad.link(&sink_pad).unwrap();
                        return;
                    }
                }
            }
        });

        let decoder = gst::ElementFactory::make(decoder_name_from_codec(info.codec))
            .name("decoder")
            .build()
            .unwrap();
        let sink = gst_app::AppSink::builder().name("sink").build();
        sink.set_sync(false);

        pipeline
            .add_many(&[
                src.upcast_ref(),
                &demuxer,
                &parser,
                &decoder,
                sink.upcast_ref(),
            ])
            .unwrap();
        gst::Element::link_many(&[src.upcast_ref(), &demuxer]).unwrap();
        gst::Element::link_many(&[&parser, &decoder, sink.upcast_ref()]).unwrap();

        pipeline.set_state(gst::State::Playing).unwrap();

        Self {
            pipeline,

            src,
            demuxer,
            parser,
            decoder,
            sink,
        }
    }

    pub(super) fn push(&self, buffer: EncodedAudioBuffer) {
        let gst_buffer = gst::Buffer::from_slice(buffer.0);

        self.src.push_buffer(gst_buffer).unwrap();
    }

    pub(super) fn pull(&self) -> Option<TimestampedRawAudioBuffer> {
        let sample = self
            .sink
            .try_pull_sample(Some(gst::ClockTime::from_mseconds(0)))?;

        let raw = raw_audio_buffer_from_sample(&sample)?;
        let (start, stop) = timestamps_from_sample(&sample)?;

        Some(TimestampedRawAudioBuffer::new(raw, start, stop))
    }

    pub(super) fn is_eos(&self) -> bool {
        self.sink.is_eos()
    }
}

fn mime_from_format(format: AudioFormat) -> &'static str {
    match format {
        AudioFormat::MpegTS => "video/mpegts",
        AudioFormat::Ogg => "audio/ogg",
        AudioFormat::Unspecified => panic!("Unsupported audio format"),
    }
}

fn mime_from_codec(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Opus => "audio/x-opus",
        AudioCodec::Vorbis => "audio/x-vorbis",
        AudioCodec::Unspecified => panic!("Unsupported audio codec"),
    }
}

fn demuxer_name_from_format(format: AudioFormat) -> &'static str {
    match format {
        AudioFormat::MpegTS => "tsdemux",
        AudioFormat::Ogg => "oggdemux",
        AudioFormat::Unspecified => panic!("Unsupported audio format"),
    }
}

fn parser_name_from_codec(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Opus => "opusparse",
        AudioCodec::Vorbis => "vorbisparse",
        AudioCodec::Unspecified => panic!("Unsupported audio codec"),
    }
}

fn decoder_name_from_codec(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Opus => "opusdec",
        AudioCodec::Vorbis => "vorbisdec",
        AudioCodec::Unspecified => panic!("Unsupported audio codec"),
    }
}

fn raw_audio_format_from_caps(caps: &gst::CapsRef) -> Option<RawAudioFormat> {
    for structure in caps.iter() {
        let Ok(str_format) = structure.get::<&str>("format") else {
            continue;
        };

        match str_format {
            "U8" => return Some(RawAudioFormat::U8),
            "S16LE" => return Some(RawAudioFormat::S16LE),
            "S16BE" => return Some(RawAudioFormat::S16BE),
            "S24LE" => return Some(RawAudioFormat::S24LE),
            "S24BE" => return Some(RawAudioFormat::S24BE),
            "S32LE" => return Some(RawAudioFormat::S32LE),
            "S32BE" => return Some(RawAudioFormat::S32BE),
            "F32LE" => return Some(RawAudioFormat::F32LE),
            "F32BE" => return Some(RawAudioFormat::F32BE),
            _ => continue,
        }
    }

    None
}

fn raw_audio_buffer_from_sample(sample: &gst::Sample) -> Option<RawAudioBuffer> {
    let caps = sample.caps()?;
    let format = raw_audio_format_from_caps(caps)?;

    let buffer = sample.buffer()?;
    let data = buffer.map_readable().ok()?.as_slice().to_vec();

    Some(RawAudioBuffer::new(data, format))
}

fn timestamps_from_sample(sample: &gst::Sample) -> Option<(Timestamp, Timestamp)> {
    let buffer = sample.buffer()?;
    let buf_start = buffer.dts_or_pts()?;
    let buf_stop = buf_start + buffer.duration()?;

    let segment = sample.segment()?;
    let gst::GenericFormattedValue::Time(Some(start)) = segment.to_running_time(buf_start) else {
        return None;
    };
    let gst::GenericFormattedValue::Time(Some(stop)) = segment.to_running_time(buf_stop) else {
        return None;
    };

    let start_ts = Timestamp::new(Duration::from_nanos(start.nseconds()));
    let stop_ts = Timestamp::new(Duration::from_nanos(stop.nseconds()));

    Some((start_ts, stop_ts))
}
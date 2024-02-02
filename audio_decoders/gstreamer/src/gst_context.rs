#[cfg(test)]
mod tests;

use core::audio_system::audio::{
    AudioCodec, EncodedAudioBuffer, EncodedAudioHeader, RawAudioBuffer, RawAudioFormat,
    TimestampedRawAudioBuffer,
};
use core::util::ClockTime;

use gst::{
    prelude::{Cast, GstBinExtManual},
    traits::ElementExt,
};
use gstreamer as gst;
use gstreamer_app as gst_app;

#[allow(dead_code)]
pub struct GstContext {
    audio_info: EncodedAudioHeader,

    pub pipeline: gst::Pipeline,

    src: gst_app::AppSrc,
    parser: gst::Element,
    decoder: gst::Element,
    convert: gst::Element,
    sink: gst_app::AppSink,
}

impl GstContext {
    pub fn new(audio_info: EncodedAudioHeader) -> Self {
        let pipeline = gst::Pipeline::new(Some("gst_audio_decoder_pipeline"));

        let src_caps = gst::Caps::builder(mime_from_codec(audio_info.codec))
            .field("rate", audio_info.sample_rate)
            .field("channels", 1)
            .build();
        let src = gst_app::AppSrc::builder()
            .name("src")
            .caps(&src_caps)
            .stream_type(gst_app::AppStreamType::Stream)
            .build();

        let parser = gst::ElementFactory::make(parser_name_from_codec(audio_info.codec))
            .name("parser")
            .build()
            .unwrap();

        let decoder = gst::ElementFactory::make(decoder_name_from_codec(audio_info.codec))
            .name("decoder")
            .build()
            .unwrap();

        let convert = gst::ElementFactory::make("audioconvert")
            .name("convert")
            .build()
            .unwrap();

        let sink_caps = gst::Caps::builder("audio/x-raw")
            .field("channels", 1)
            .build();
        let sink = gst_app::AppSink::builder()
            .name("sink")
            .caps(&sink_caps)
            .build();
        sink.set_sync(false);

        pipeline
            .add_many(&[src.upcast_ref(), &parser, &decoder, &convert, sink.upcast_ref()])
            .unwrap();
        gst::Element::link_many(&[src.upcast_ref(), &parser,
            &decoder, &convert, sink.upcast_ref()]).unwrap();

        let this = Self {
            audio_info,

            pipeline,

            src,
            parser,
            decoder,
            convert,
            sink,
        };
        this.make_playing();

        this
    }

    pub fn push(&self, buffer: EncodedAudioBuffer) {
        let mut gst_buffer = gst::Buffer::from_slice(buffer.data);

        if let Some(gst_buffer) = gst_buffer.get_mut() {
            if let Some(ts) = buffer.start_ts {
                let ts = gst::ClockTime::from_nseconds(ts.as_nanos());

                gst_buffer.set_dts(ts);
                gst_buffer.set_pts(ts);
            }
        }

        let _ = self.src.push_buffer(gst_buffer);
    }

    pub fn pull(&self) -> Option<TimestampedRawAudioBuffer> {
        let sample = self
            .sink
            .try_pull_sample(Some(gst::ClockTime::from_mseconds(1)))?;

        let raw = raw_audio_buffer_from_sample(&sample, self.audio_info)?;
        let start = timestamps_from_sample(&sample);

        Some(TimestampedRawAudioBuffer::new(raw, start))
    }

    pub fn push_eos(&self) {
        let _ = self.src.end_of_stream();
    }

    pub fn is_eos(&self) -> bool {
        self.sink.is_eos()
    }

    pub fn make_playing(&self) {
        self.pipeline.set_state(gst::State::Playing).unwrap();
    }

    pub fn make_null(&self) {
        self.pipeline.set_state(gst::State::Null).unwrap();
    }

    pub fn is_playing_failed(&self) -> bool {
        let (res, _, pending) = self.pipeline.state(Some(gst::ClockTime::from_mseconds(0)));

        res.is_err() && pending == gst::State::Playing
    }
}

fn mime_from_codec(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Opus => "audio/x-opus",
        AudioCodec::Unspecified => panic!("Unsupported audio codec"),
    }
}

fn parser_name_from_codec(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Opus => "opusparse",
        AudioCodec::Unspecified => panic!("Unsupported audio codec"),
    }
}

fn decoder_name_from_codec(codec: AudioCodec) -> &'static str {
    match codec {
        AudioCodec::Opus => "opusdec",
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

fn raw_audio_buffer_from_sample(
    sample: &gst::Sample,
    audio_info: EncodedAudioHeader,
) -> Option<RawAudioBuffer> {
    let caps = sample.caps()?;
    let format = raw_audio_format_from_caps(caps)?;

    let buffer = sample.buffer()?;
    let data = buffer.map_readable().ok()?.as_slice().to_vec();

    Some(RawAudioBuffer::new(data, format, audio_info.sample_rate))
}

fn timestamps_from_sample(sample: &gst::Sample) -> Option<ClockTime> {
    let Some(buffer) = sample.buffer() else {
        return None;
    };
    let Some(mut buf_start) = buffer.dts_or_pts() else {
        return None;
    };
    let Some(segment) = sample.segment() else {
        return Some(to_custom_clock_time(buf_start));
    };

    buf_start = to_running_time(segment, buf_start);
    let start_ts = to_custom_clock_time(buf_start);

    Some(start_ts)
}

fn to_custom_clock_time(ts: gst::ClockTime) -> ClockTime {
    ClockTime::from_nanos(ts.nseconds())
}

fn to_running_time(segment: &gst::Segment, ts: gst::ClockTime) -> gst::ClockTime {
    let gst::GenericFormattedValue::Time(Some(start)) = segment.start() else {
        return ts;
    };
    let gst::GenericFormattedValue::Time(Some(offset)) = segment.offset() else {
        return ts - start;
    };
    let gst::GenericFormattedValue::Time(Some(base)) = segment.base() else {
        return ts - start - offset;
    };

    ts - start - offset + base
}

use crate::{audio_stream::AudioStream, message_stream::MessageStream};
use core::error;
use std::time::Duration;

use mio::*;

const MESSAGE: Token = Token(2);
const AUDIO: Token = Token(3);

pub(super) struct Poller {
    poll: Poll,
    events: Events,
}

impl Poller {
    pub(super) fn new() -> error::Result<Self> {
        Ok(Self {
            poll: Poll::new()?,
            events: Events::with_capacity(32),
        })
    }

    pub(super) fn register_message_stream(
        &mut self,
        msg_stream: &mut MessageStream,
    ) -> error::Result<()> {
        self.poll.registry().register(
            msg_stream.socket_mut(),
            MESSAGE,
            Interest::READABLE | Interest::WRITABLE,
        )?;

        Ok(())
    }

    pub(super) fn deregister_message_stream(
        &mut self,
        msg_stream: &mut MessageStream,
    ) -> error::Result<()> {
        self.poll.registry().deregister(msg_stream.socket_mut())?;

        Ok(())
    }

    pub(super) fn register_audio_stream(
        &mut self,
        audio_stream: &mut AudioStream,
    ) -> error::Result<()> {
        self.poll
            .registry()
            .register(audio_stream.socket_mut(), AUDIO, Interest::READABLE)?;

        Ok(())
    }

    pub(super) fn deregister_audio_stream(
        &mut self,
        audio_stream: &mut AudioStream,
    ) -> error::Result<()> {
        self.poll.registry().deregister(audio_stream.socket_mut())?;

        Ok(())
    }

    pub(super) fn poll(
        &mut self,
        message_stream: &mut MessageStream,
        mut audio_stream: Option<&mut AudioStream>,
    ) -> error::Result<()> {
        self.poll
            .poll(&mut self.events, Some(Duration::from_millis(0)))?;
        for e in self.events.iter() {
            match e.token() {
                MESSAGE => {
                    if e.is_readable() {
                        let _ = message_stream.recv_to_buf();
                    }

                    if e.is_writable() {
                        let _ = message_stream.send_from_buf();
                    }
                }
                AUDIO => {
                    let audio_stream = match &mut audio_stream {
                        Some(audio_stream) => &mut **audio_stream,
                        None => continue,
                    };

                    if e.is_readable() {
                        audio_stream.recv_to_buf();
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
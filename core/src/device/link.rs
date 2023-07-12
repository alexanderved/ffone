use std::net::SocketAddr;

use super::element::*;
use super::DeviceInfo;

use crate::error;
use crate::util::*;

use mueue::{Message, MessageEndpoint};

pub type DeviceLinkEndpoint = MessageEndpoint<DeviceLinkControlMessage, DeviceLinkMessage>;
pub type DeviceLinkStateMachine = RunnableStateMachine<Box<dyn DeviceLink>>;

#[derive(Debug)]
#[non_exhaustive]
pub enum DeviceLinkMessage {
    Info(DeviceInfo),

    AudioAddress(SocketAddr),
    GetAudioAddressError(error::Error),

    DeviceDisconnected,
}

impl Message for DeviceLinkMessage {}

#[derive(Debug)]
#[non_exhaustive]
pub enum DeviceLinkControlMessage {
    GetInfo,
    GetAudioAddress,

    Stop,
}

impl Message for DeviceLinkControlMessage {}

#[rustfmt::skip]
pub trait DeviceLink: DeviceSystemElement + Send {
    fn info(&self) -> DeviceInfo;
    fn audio_address(&self) -> Option<SocketAddr>;
}

/* crate::impl_control_message_handler! {
    @component DeviceLink;
    @message DeviceLinkMessage;
    @control_message DeviceLinkControlMessage;

    GetInfo => info => Info;
    GetAudioAddress => audio_address => @ok AudioAddress, @err GetAudioAddressError;
} */

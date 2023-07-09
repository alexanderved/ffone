use std::net::SocketAddr;

use super::DeviceInfo;

use crate::error;
use crate::util::*;

use mueue::{Message, MessageEndpoint};

pub type DeviceLinkEndpoint = MessageEndpoint<DeviceLinkControlMessage, DeviceLinkMessage>;

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

pub trait DeviceLink:
    Component<Message = DeviceLinkMessage, ControlMessage = DeviceLinkControlMessage>
    + Runnable
    + Send
    + Sync
{
    fn info(&self) -> DeviceInfo;
    fn audio_address(&self) -> error::Result<SocketAddr>;
}

crate::impl_control_message_handler! {
    @component DeviceLink;
    @message DeviceLinkMessage;
    @control_message DeviceLinkControlMessage;

    GetInfo => info => Info;
    GetAudioAddress => audio_address => @map_or_else(GetAudioAddressError, AudioAddress);
}

use super::*;

use crate::audio_system::audio::MuxedAudioBuffer;
use crate::error;
use crate::util::*;

use mueue::*;

#[non_exhaustive]
pub enum DeviceSystemElementMessage {
    NewDevicesDiscovered(Box<dyn Iterator<Item = DeviceInfo> + Send + Sync>),
    DeviceUnreachable(DeviceInfo),

    LinkedDeviceInfo(DeviceInfo),

    MuxedAudioReceived(MuxedAudioBuffer),

    DeviceUnlinked,

    Error(error::Error),
}

impl Message for DeviceSystemElementMessage {}

crate::trait_alias!(@upcast AsDeviceSystemElement pub DeviceSystemElement:
    Element<Message = DeviceSystemElementMessage> + Runnable);
crate::impl_as_trait!(device_system_element -> DeviceSystemElement);

pub trait DeviceSystemElementBuilder: ElementBuilder
where
    Self::Element: DeviceSystemElement,
{
}

impl<B: ElementBuilder> DeviceSystemElementBuilder for B where Self::Element: DeviceSystemElement {}

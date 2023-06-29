use std::net::SocketAddr;

use mueue::{Message, MessageEndpoint};

use crate::{Runnable, Component};

type DeviceLinkEndpoint = MessageEndpoint<DeviceControlMessage, DeviceMessage>;

pub enum DeviceMessage {}

impl Message for DeviceMessage {}

pub enum DeviceControlMessage {}

impl Message for DeviceControlMessage {}

pub trait DeviceLink: Component + Runnable {}

use crate::util::{Component, Runnable};

use mueue::Message;

pub enum DeviceMessage {}

impl Message for DeviceMessage {}

pub enum DeviceControlMessage {}

impl Message for DeviceControlMessage {}

pub trait DeviceLink: Component + Runnable {}

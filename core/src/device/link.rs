use super::element::*;
use super::DeviceInfo;

use crate::util::*;

pub type DeviceLinkStateMachine = RunnableStateMachine<Box<dyn DeviceLink>>;

pub trait DeviceLink: DeviceSystemElement + Send {
    fn info(&self) -> DeviceInfo;
}

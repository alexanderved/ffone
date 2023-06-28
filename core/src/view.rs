use crate::*;

use mueue::Message;

pub enum ViewRequest {}

impl Message for ViewRequest {}

pub enum ViewControlMessage {}

impl Message for ViewControlMessage {}

pub trait View:
    Component<Message = ViewRequest, ControlMessage = ViewControlMessage> + Runnable
{
}

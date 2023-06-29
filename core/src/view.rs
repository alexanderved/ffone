use crate::util::*;

use mueue::Message;

pub enum ViewMessage {}

impl Message for ViewMessage {}

pub enum ViewControlMessage {}

impl Message for ViewControlMessage {}

pub trait View:
    Component<Message = ViewMessage, ControlMessage = ViewControlMessage> + Runnable
{
}

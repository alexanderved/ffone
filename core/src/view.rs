use crate::util::*;

use mueue::{Message, MessageEndpoint};

pub type ViewEndpoint = MessageEndpoint<ViewControlMessage, ViewMessage>;

#[non_exhaustive]
pub enum ViewMessage {}

impl Message for ViewMessage {}

#[non_exhaustive]
pub enum ViewControlMessage {
    Stop,
}

impl Message for ViewControlMessage {}

pub trait View:
    Component<Message = ViewMessage, ControlMessage = ViewControlMessage> + Runnable
{
}

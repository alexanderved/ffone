use crate::{*, controller::ViewControlMessage};

use mueue::Message;

pub enum ViewRequest {}

impl Message for ViewRequest {}

pub trait View:
    Component<Message = ViewRequest, ControlMessage = ViewControlMessage> + Runnable
{
}

pub mod discoverer;
pub mod element;
pub mod link;

use discoverer::*;
use element::*;
use link::*;
use mueue::*;

use crate::util::{Component, Runnable};

use std::collections::HashMap;

pub type DeviceSystemEndpoint = MessageEndpoint<DeviceSystemControlMessage, DeviceSystemMessage>;

pub enum DeviceSystemMessage {}

impl Message for DeviceSystemMessage {}

pub enum DeviceSystemControlMessage {}

impl Message for DeviceSystemControlMessage {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub name: String,
}

impl DeviceInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
        }
    }
}

pub struct DeviceSystem {
    end: DeviceSystemEndpoint,
    notification_recv: MessageReceiver<DeviceSystemElementMessage>,

    active_discoverer: Option<DeviceDiscovererStateMachine>,
    discoverers: HashMap<DeviceDiscovererInfo, Option<Box<dyn DeviceDiscoverer>>>,

    link: Option<DeviceLinkStateMachine>,
}

impl DeviceSystem {
    pub fn new(
        end: DeviceSystemEndpoint,
        discoverers_builders: Vec<Box<dyn DeviceDiscovererBuilder>>,
    ) -> Self {
        let (notification_send, notification_recv) = unidirectional_queue();
        let discoverers = collect_discoverers(discoverers_builders, notification_send);

        Self {
            end,
            notification_recv,

            active_discoverer: None,
            discoverers,

            link: None,
        }
    }
}

impl Component for DeviceSystem {
    type Message = DeviceSystemMessage;
    type ControlMessage = DeviceSystemControlMessage;

    fn endpoint(&self) -> MessageEndpoint<Self::ControlMessage, Self::Message> {
        self.end.clone()
    }

    fn connect(&mut self, end: MessageEndpoint<Self::ControlMessage, Self::Message>) {
        self.end = end;
    }
}

impl Runnable for DeviceSystem {
    fn update(&mut self) -> crate::error::Result<()> {
        Ok(())
    }
}

fn collect_discoverers(
    discoverers_builders: Vec<Box<dyn DeviceDiscovererBuilder>>,
    sender: MessageSender<DeviceSystemElementMessage>,
) -> HashMap<DeviceDiscovererInfo, Option<Box<dyn DeviceDiscoverer>>> {
    discoverers_builders
        .into_iter()
        .map(|mut builder| {
            builder.set_sender(sender.clone());
            builder
        })
        .filter_map(|builder| builder.build().ok())
        .map(|disc| (disc.info(), Some(disc)))
        .collect()
}

use std::{any::TypeId, collections::HashMap, sync::mpsc::Sender};

use serde::de::DeserializeOwned;

use crate::{
    Maelstrom,
    messages::{
        Body, Broadcast, BroadcastOk, Echo, EchoOk, Generate, GenerateOk, Init, InitOk, Message,
        Read, ReadOk, Topology, TopologyOk,
    },
};

type HandlerFn<U> =
    dyn Fn(&Body, &mut Sender<Message>, &str, &mut Maelstrom, &mut U) + Send + 'static;

#[derive(Default)]
pub struct Router<U> {
    handlers: HashMap<TypeId, Box<HandlerFn<U>>>,
    tick: Option<Box<dyn Fn(&mut Sender<Message>, &mut Maelstrom, &mut U) + Send + 'static>>,
}

impl<U> Router<U> {
    pub fn on<M, F>(&mut self, handler: F)
    where
        M: 'static + DeserializeOwned,
        Body: Into<M> + Clone,
        F: Fn(M, &mut Sender<Message>, &str, &mut Maelstrom, &mut U) + Send + 'static,
    {
        let key = TypeId::of::<M>();
        let handler = move |body: &Body,
                            tx_output: &mut Sender<Message>,
                            src: &str,
                            maelstrom: &mut Maelstrom,
                            user_data: &mut U| {
            let m: M = body.clone().into();
            handler(m, tx_output, src, maelstrom, user_data)
        };
        self.handlers.insert(key, Box::new(handler));
    }

    pub fn set_tick<F>(&mut self, handler: F)
    where
        F: Fn(&mut Sender<Message>, &mut Maelstrom, &mut U) + Send + 'static,
    {
        self.tick = Some(Box::new(handler));
    }

    pub fn tick(
        &self,
        tx_output: &mut Sender<Message>,
        maelstrom_data: &mut Maelstrom,
        user_data: &mut U,
    ) {
        if let Some(tick) = &self.tick {
            tick(tx_output, maelstrom_data, user_data);
        }
    }

    pub fn handle(
        &self,
        msg: Message,
        tx_output: &mut Sender<Message>,
        maelstrom_data: &mut Maelstrom,
        user_data: &mut U,
    ) {
        let key = match &msg.body {
            Body::Init(_) => TypeId::of::<Init>(),
            Body::InitOk(_) => TypeId::of::<InitOk>(),
            Body::Echo(_) => TypeId::of::<Echo>(),
            Body::EchoOk(_) => TypeId::of::<EchoOk>(),
            Body::Generate(_) => TypeId::of::<Generate>(),
            Body::GenerateOk(_) => TypeId::of::<GenerateOk>(),
            Body::Topology(_) => TypeId::of::<Topology>(),
            Body::TopologyOk(_) => TypeId::of::<TopologyOk>(),
            Body::Broadcast(_) => TypeId::of::<Broadcast>(),
            Body::BroadcastOk(_) => TypeId::of::<BroadcastOk>(),
            Body::Read(_) => TypeId::of::<Read>(),
            Body::ReadOk(_) => TypeId::of::<ReadOk>(),
        };

        if let Some(handler) = self.handlers.get(&key) {
            handler(&msg.body, tx_output, &msg.src, maelstrom_data, user_data);
        }
    }
}

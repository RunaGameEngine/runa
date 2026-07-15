use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait Event: 'static {}
type EventCallback = Box<dyn Fn(&dyn Any)>;
pub struct EventBus {
    pub listeners: HashMap<TypeId, Vec<EventCallback>>,
    pub queue: Vec<Box<dyn Any>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            queue: vec![],
        }
    }

    pub fn emit<E: 'static>(&mut self, event: E) {
        self.queue.push(Box::new(event));
    }

    pub fn subscribe<E: 'static>(&mut self, callback: impl Fn(&E) + 'static) {
        let type_id = TypeId::of::<E>();

        let wrapped_callback: EventCallback = Box::new(move |event| {
            if let Some(event) = event.downcast_ref::<E>() {
                callback(event);
            }
        });

        self.listeners
            .entry(type_id)
            .or_default()
            .push(wrapped_callback);
    }

    pub fn process(&mut self) {
        let events = std::mem::take(&mut self.queue);

        for event in events {
            let type_id = (*event).type_id();

            if let Some(callbacks) = self.listeners.get(&type_id) {
                for callback in callbacks {
                    callback(event.as_ref());
                }
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub trait Event: Send + 'static {}
type EventCallback = Box<dyn Fn(&dyn Any) + Send>;

struct EventBusInner {
    listeners: HashMap<TypeId, Vec<EventCallback>>,
    queue: Vec<Box<dyn Any + Send>>,
}

impl EventBusInner {
    fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            queue: Vec::new(),
        }
    }

    fn emit<E: Event>(&mut self, event: E) {
        self.queue.push(Box::new(event));
    }

    fn subscribe<E: Event>(&mut self, callback: impl Fn(&E) + Send + 'static) {
        let type_id = TypeId::of::<E>();
        let wrapped: EventCallback = Box::new(move |event| {
            if let Some(e) = event.downcast_ref::<E>() {
                callback(e);
            }
        });
        self.listeners.entry(type_id).or_default().push(wrapped);
    }

    fn process(&mut self) {
        let events = std::mem::take(&mut self.queue);
        for event in events {
            let tid = (*event).type_id();
            if let Some(callbacks) = self.listeners.get(&tid) {
                for cb in callbacks {
                    cb(event.as_ref());
                }
            }
        }
    }
}

static EVENT_BUS: OnceLock<Mutex<EventBusInner>> = OnceLock::new();

fn global() -> &'static Mutex<EventBusInner> {
    EVENT_BUS.get_or_init(|| Mutex::new(EventBusInner::new()))
}

/// Global event bus (singleton, like `InputState` / `AudioEngine`).
///
/// No need to spawn anything or query the world — just emit, subscribe,
/// and process from anywhere.
///
/// ```ignore
/// EventBus::emit(MyEvent { x: 1 });
/// EventBus::subscribe(|e: &MyEvent| println!("got {}", e.x));
/// EventBus::process(); // dispatch queued events to subscribers
/// ```
pub struct EventBus;

impl EventBus {
    /// Queue an event for the next `process()` call.
    pub fn emit<E: Event>(event: E) {
        global().lock().unwrap().emit(event);
    }

    /// Register a callback for a given event type.
    pub fn subscribe<E: Event>(callback: impl Fn(&E) + Send + 'static) {
        global().lock().unwrap().subscribe(callback);
    }

    /// Drain the event queue and dispatch to all matching subscribers.
    pub fn process() {
        global().lock().unwrap().process();
    }
}

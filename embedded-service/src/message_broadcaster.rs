//! Message Broadcasting Interface
//!
//! This module provides a way to broadcast messages to multiple different services. The intended use is for a service
//! to create an instance of a [`Broadcaster`] and provide an interface for other services to take
//! publishers/subscribers. The [`Broadcaster`] instance (and underlying [`PubSubChannel`]) can be held in other
//! services.

use core::{fmt::{self, Debug}, marker::PhantomData};

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel, signal::Signal};
use defmt::error;

use crate::{intrusive_list, IntrusiveList, Node, NodeContainer};

pub trait Listen<T: Clone> : NodeContainer +  Debug {
    fn handle_message(&self, message: &T);
    fn is_empty(&self) -> bool;
    async fn listen(&self) -> T;
}

pub struct MessageBroadcaster<T> {
    listeners: IntrusiveList,
    // TODO(melvin): can the intrusive list have a generic variant so this phantom data is not needed?
    _marker: PhantomData<T>,
}

#[derive(Debug)]
pub enum Listener<T: Clone + Debug + 'static> {
    Signal(SignalListener<T>),
    Channel(ChannelListener<T>),
    // Custom(&'static dyn Listen<T>),
}

pub struct SignalListener<T: Clone + Debug> {
    signal: Signal<NoopRawMutex, T>,
    node: Node,
}

pub struct ChannelListener<T: Clone + Debug> {
    // TODO(melvin): let client choose channel size
    channel: Channel<NoopRawMutex, T, 20>,
    node: Node,
}

impl<T: Clone + Debug + 'static> MessageBroadcaster<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_listener(&self, object: &'static Listener<T>) -> Result<(), intrusive_list::Error>{
        // TODO(melvin): can this can use a variant of IntrusiveList that is generic without dynamic dispatch?
        
        self.listeners.push(object)
    }

    pub fn broadcast(&self, message: T) {
        for listener_node in &self.listeners {
            if let Some(listener) = listener_node.data::<Listener<T>>() {
                // TODO(melvin): Insert some flow control logic here to backoff if any of the listeners are full. we can defer the behavior upwards to the caller, so that caller can decide what to do
                listener.handle_message(&message);
            }
        }
    }
}

impl<T> Default for MessageBroadcaster<T> {
    fn default() -> Self {
        Self {
            listeners: IntrusiveList::new(),
            _marker: PhantomData,
        }
    }
}

impl <T: Clone + Debug + 'static> NodeContainer for Listener<T> {
    fn get_node(&self) -> &Node {
        match self {
            Listener::Signal(signal_listener) => signal_listener.get_node(),
            Listener::Channel(channel_listener) => channel_listener.get_node(),
            // Listener::Custom(custom_listener) => custom_listener.get_node(),
        }
    }
}

impl <T: Clone + Debug + 'static> Listen<T> for Listener<T> {
    fn handle_message(&self, message: &T) {
        match self {
            Listener::Signal(signal_listener) => signal_listener.handle_message(message),
            Listener::Channel(channel_listener) => channel_listener.handle_message(message),
            // Listener::Custom(custom_listener) => custom_listener.handle_message(message),
        }
    }
    
    async fn listen(&self) -> T {
        match self {
            Listener::Signal(signal_listener) => signal_listener.listen().await,
            Listener::Channel(channel_listener) => channel_listener.listen().await,
            // Listener::Custom(custom_listener) => custom_listener.listen().await,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Listener::Signal(signal_listener) => signal_listener.is_empty(),
            Listener::Channel(channel_listener) => channel_listener.is_empty(),
            // Listener::Custom(custom_listener) => custom_listener.is_empty(),
        }
    }
}

impl<T: Clone + Debug + 'static> SignalListener<T> {
    pub fn new() -> Self {
        Self {
            signal: Signal::new(),
            node: Node::uninit(),
        }
    }
}

impl<T: Clone + Debug> Debug for SignalListener<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignalListener {
                signal,
                node,
            } => {
                f.debug_struct("SignalListener")
                    .field("signal", &signal.signaled())
                    .field("node", node)
                    .finish()
            }
        }
    }
}

impl<T: Clone + Debug + 'static> NodeContainer for SignalListener<T> {
    fn get_node(&self) -> &Node {
        &self.node
    }
}

impl<T: Clone + Debug + 'static> Listen<T> for SignalListener<T> {
    fn handle_message(&self, message: &T) {
        if self.signal.signaled() {
            self.signal.signal(message.clone());
        } else {
            error!("signal listener already signaled!");
            // TODO(melvin): Signal already present. Listener full. propagate full error up to caller to tell it to backoff?
        }
    }
    
    async fn listen(&self) -> T {
        self.signal.wait().await
    }
    
    fn is_empty(&self) -> bool {
        !self.signal.signaled()
    }

    
}

impl<T: Clone + Debug + 'static> ChannelListener<T> {
    pub fn new() -> Self {
        Self {
            channel: Channel::new(),
            node: Node::uninit(),
        }
    }
}

impl<T: Clone + Debug> Debug for ChannelListener<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelListener {
                channel,
                node,
            } => {
                f.debug_struct("ChannelListener")
                    .field("channel(len)", &channel.len())
                    .field("node", node)
                    .finish()
            }
        }
    }
}

impl<T: Clone + Debug + 'static> NodeContainer for ChannelListener<T> {
    fn get_node(&self) -> &Node {
        &self.node
    }
}

impl<T: Clone + Debug + 'static> Listen<T> for ChannelListener<T> {
    fn handle_message(&self, message: &T) {
        match self.channel.try_send(message.clone()) {
            Ok(()) => return,
            Err(_) => {
                error!("channel listener already full!");
                // TODO(melvin): Channel is full. Propagate error up to caller to tell it to backoff?
            }
        }
    }
    
    async fn listen(&self) -> T {
        self.channel.receive().await
    }
    
    fn is_empty(&self) -> bool {
        self.channel.is_empty()
    }
}

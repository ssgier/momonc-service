use futures::stream::SplitSink;
use fxhash::FxHashMap;
use tokio::{
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::{app_state::AppEvent, domain::StatusMessage};

pub type EventSender = UnboundedSender<AppEvent>;
pub type EventReceiver = UnboundedReceiver<AppEvent>;
pub type StatusSender = UnboundedSender<StatusMessage>;
pub type StatusReceiver = UnboundedReceiver<StatusMessage>;
pub type OutSink = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type AppHashMap<K, V> = FxHashMap<K, V>;

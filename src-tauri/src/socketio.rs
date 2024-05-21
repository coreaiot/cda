use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::HashMap, io};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{self, sleep};
use futures_util::FutureExt;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder}
};

// fn get_sockets() -> &'static mut HashMap<String, UdpSocket> {
//     static HASHMAP: OnceLock<HashMap<String, UdpSocket>> = OnceLock::new();
//     HASHMAP.get_or_init(|| {
//       let mut m = HashMap::new();
//       m
//     })
// }

struct SocketIO {
    pub task: JoinHandle<()>,
    pub sock: Arc<Client>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Payload {
    addr: String,
    data: Vec<u8>,
}

lazy_static! {
    static ref SOCKETS: RwLock<HashMap<String, Client>> = RwLock::new(HashMap::new());
}

pub async fn connect(window: tauri::Window, id: String, addr: String) -> io::Result<()> {
    let mut sockets = SOCKETS.write().await;

    if let Some(s) = sockets.get(&id) {
        s.disconnect().await;
        sockets.remove(&id);
        sleep(time::Duration::from_millis(100)).await;
    }

    let sock = ClientBuilder::new(&addr)
        .namespace("/admin")
        .on("error", |err, _| {
            async move { eprintln!("Error: {:#?}", err) }.boxed()
        })
        .connect()
        .await?;
    println!("{} socketio connected to {}", &id, &addr);

    sockets.insert(id, sock);
    Ok(())
}

pub async fn disconnect(id: String) -> bool {
    let mut sockets = SOCKETS.write().await;

    if let Some(s) = sockets.get(&id) {
        s.disconnect().await;
        sockets.remove(&id);
        println!("{} socketio disconnected", &id);
        true
    } else {
        false
    }
}

pub async fn send(id: String, topic: String, message: Vec<u8>) -> bool {
    let sockets = SOCKETS.read().await;

    if let Some(s) = sockets.get(&id) {
        println!("{} socketio send {} bytes", &id, message.len());
        match s.emit(topic, message).await {
            Ok(_) => true,
            Err(_) => false,
        }
    } else {
        false
    }
}

pub async fn subscribe(id: String, topic: String) -> bool {
    let mut sockets = SOCKETS.write().await;

    if let Some(s) = sockets.get(&id) {
        println!("{} socketio subscribe {}", &id, topic);
        match s.emit(topic, message).await {
            Ok(_) => true,
            Err(_) => false,
        }
    } else {
        false
    }
}

#[tauri::command]
pub async fn udp_bind(window: tauri::Window, id: String, bind_at: String) -> bool {
    match bind(window, id, bind_at).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub async fn udp_unbind(id: String) -> bool {
    unbind(id).await
}

#[tauri::command]
pub async fn udp_send(id: String, target: String, message: Vec<u8>) -> bool {
    println!("{} udp send {} bytes to {}", &id, message.len(), &target);
    send(id, target, message).await
}

// server/main.rs
use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, tcp::OwnedWriteHalf};
use tokio::sync::Mutex;

use messages::{ClientMessage, PlayerPosition, ServerMessage};

struct Server {
    client_map: BTreeMap<u32, Arc<Mutex<Client>>>,
}

impl Server {
    fn new() -> Self {
        Server {
            client_map: BTreeMap::new(),
        }
    }

    async fn handle_message(
        &mut self,
        client: &Arc<Mutex<Client>>,
        msg: ClientMessage,
        server_arc: Arc<Mutex<Server>>,
    ) {
        let pid = client.lock().await.player_id;
        match msg {
            ClientMessage::Move { position } => {
                let move_msg = ServerMessage::PlayerMove {
                    moved_player_id: pid,
                    position,
                };
                for (&other_pid, other_client) in &self.client_map {
                    if other_pid != pid {
                        other_client.lock().await.send(move_msg.clone(), server_arc.clone()).await;
                    }
                }
            }
            _ => { /* ... */ }
        }
    }

    async fn add_client(&mut self, c: Arc<Mutex<Client>>) {
        let pid = c.lock().await.player_id;
        self.client_map.insert(pid, c);
    }

    async fn remove_client(&mut self, pid: u32) {
        self.client_map.remove(&pid);
    }
}

struct Client {
    player_id: u32,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    buffer: Arc<Mutex<VecDeque<ServerMessage>>>,
    sending: Arc<Mutex<bool>>,
}

impl Client {
    fn new(player_id: u32, writer: OwnedWriteHalf) -> Self {
        Client {
            player_id,
            writer: Arc::new(Mutex::new(writer)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            sending: Arc::new(Mutex::new(false)),
        }
    }

    async fn send(&mut self, msg: ServerMessage, server_arc: Arc<Mutex<Server>>) {
        self.buffer.lock().await.push_back(msg);
        let mut sending_guard = self.sending.lock().await;
        if !*sending_guard {
            *sending_guard = true;
            let buffer_arc = self.buffer.clone();
            let writer_arc = self.writer.clone();
            let sending_arc = self.sending.clone();
            let my_pid = self.player_id;
            let _ = tokio::spawn(async move {
                while let Some(m) = buffer_arc.lock().await.pop_front() {
                    let bytes = bincode::serialize(&m).unwrap();
                    if writer_arc.lock().await.write_all(&bytes).await.is_err() {
                        server_arc.lock().await.remove_client(my_pid).await;
                        break;
                    }
                }
                *sending_arc.lock().await = false;
            });
        }
    }
}

#[tokio::main]
async fn main() {
    let port = env::args().nth(1).unwrap_or_else(|| "4242".to_string());
    let listener = TcpListener::bind(("0.0.0.0", port.parse::<u16>().unwrap()))
        .await
        .unwrap();

    println!("Server running on port {} ...", port);

    let server_arc = Arc::new(Mutex::new(Server::new()));
    let mut last_pid = 0;

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let pid = last_pid;
        last_pid += 1;

        let server_clone = server_arc.clone();
        tokio::spawn(async move {
            handle_client(socket, pid, server_clone).await;
        });
    }
}

async fn handle_client(socket: tokio::net::TcpStream, pid: u32, server_arc: Arc<Mutex<Server>>) {
    let (mut reader, writer) = socket.into_split();
    let client = Arc::new(Mutex::new(Client::new(pid, writer)));

    {
        let mut s = server_arc.lock().await;
        s.add_client(client.clone()).await;
    }
    {
        let mut c = client.lock().await;
        let init_msg = ServerMessage::Init {
            your_player_id: pid,
            your_position: PlayerPosition::NotInWorld,
        };
        c.send(init_msg, server_arc.clone()).await;
    }
    println!("Player {} connected", pid);

    let mut buf = Vec::new();
    while let Ok(n) = reader.read_buf(&mut buf).await {
        if n == 0 { break; }
        while let Ok((msg, consumed)) = try_deser::<ClientMessage>(&buf) {
            buf.drain(..consumed);
            let mut s = server_arc.lock().await;
            s.handle_message(&client, msg, server_arc.clone()).await;
        }
    }
    {
        let mut s = server_arc.lock().await;
        s.remove_client(pid).await;
    }
    println!("Player {} disconnected", pid);
}

fn try_deser<T: serde::de::DeserializeOwned>(
    buf: &[u8],
) -> Result<(T, usize), bincode::Error> {
    let mut cur = std::io::Cursor::new(buf);
    match bincode::deserialize_from(&mut cur) {
        Ok(m) => Ok((m, cur.position() as usize)),
        Err(_) => Err(bincode::ErrorKind::SizeLimit.into()),
    }
}

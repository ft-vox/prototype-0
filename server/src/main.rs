use messages::{ClientMessage, PlayerPosition, ServerMessage};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::env;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{tcp::OwnedWriteHalf, TcpListener};
use tokio::sync::Mutex;

type ChunkIndex = (i32, i32);

struct Server {
    client_map: BTreeMap<u32, Arc<Mutex<Client>>>,
    watchers: Arc<Mutex<HashMap<ChunkIndex, Arc<Mutex<HashSet<u32>>>>>>,
}

struct Client {
    player_id: u32,
    watching_chunks: Arc<Mutex<HashSet<ChunkIndex>>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    buffer: Arc<Mutex<VecDeque<ServerMessage>>>,
    is_sender_spawned: Arc<Mutex<bool>>,
}

impl Server {
    fn new() -> Self {
        Server {
            client_map: BTreeMap::new(),
            watchers: Arc::new(Mutex::new(HashMap::new())),
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
                        other_client
                            .lock()
                            .await
                            .send(move_msg.clone(), server_arc.clone())
                            .await;
                    }
                }
            }
            ClientMessage::WatchChunk { x, y } => {
                let index: ChunkIndex = (x, y);
                let Some(tmp) = self.client_map.get_mut(&pid) else {
                    return;
                };
                let player = tmp.lock().await;
                let newly_added = player.watching_chunks.lock().await.insert(index);
                if newly_added {
                    let mut watcher = self.watchers.lock().await;
                    let entry = watcher
                        .entry(index)
                        .or_insert_with(|| Arc::new(Mutex::new(HashSet::new())));
                    let mut set = entry.lock().await;
                    set.insert(pid);
                }
            }
            ClientMessage::UnwatchChunk { x, y } => {
                let index: ChunkIndex = (x, y);
                let Some(tmp) = self.client_map.get_mut(&pid) else {
                    return;
                };
                let player = tmp.lock().await;
                let deleted = player.watching_chunks.lock().await.remove(&index);
                if deleted {
                    let mut watcher = self.watchers.lock().await;
                    let to_delete = {
                        let set_arc = watcher.get(&index).unwrap();
                        let mut set = set_arc.lock().await;
                        set.remove(&pid);
                        set.is_empty()
                    };
                    if to_delete {
                        watcher.remove(&index);
                    }
                }
            }
            _ => {
                // TODO: remove _
            }
        };
    }

    async fn add_client(&mut self, c: Arc<Mutex<Client>>) {
        let pid = c.lock().await.player_id;
        self.client_map.insert(pid, c);
    }

    async fn remove_client(&mut self, player_id: u32) {
        let tmp = self.client_map.remove(&player_id).unwrap();
        let client = tmp.lock().await;
        let mut watchers = self.watchers.lock().await;
        for index in client.watching_chunks.lock().await.iter() {
            let to_delete = {
                let tmp = watchers.get(index).unwrap();
                let mut node = tmp.lock().await;
                node.remove(&client.player_id);
                node.len() == 0
            };
            if to_delete {
                watchers.remove(index);
            }
        }
    }
}

impl Client {
    fn new(player_id: u32, writer: OwnedWriteHalf) -> Self {
        Client {
            player_id,
            watching_chunks: Arc::new(Mutex::new(HashSet::new())),
            writer: Arc::new(Mutex::new(writer)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            is_sender_spawned: Arc::new(Mutex::new(false)),
        }
    }

    async fn send(&mut self, msg: ServerMessage, server_arc: Arc<Mutex<Server>>) {
        self.buffer.lock().await.push_back(msg);
        let mut is_sender_spawned_guard = self.is_sender_spawned.lock().await;
        if !*is_sender_spawned_guard {
            *is_sender_spawned_guard = true;
            let buffer_arc = self.buffer.clone();
            let writer_arc = self.writer.clone();
            let is_sender_spawned_arc = self.is_sender_spawned.clone();
            let my_pid = self.player_id;
            tokio::spawn(async move {
                while let Some(m) = buffer_arc.lock().await.pop_front() {
                    let bytes = bincode::serialize(&m).unwrap();
                    if writer_arc.lock().await.write_all(&bytes).await.is_err() {
                        server_arc.lock().await.remove_client(my_pid).await;
                        break;
                    }
                }
                *is_sender_spawned_arc.lock().await = false;
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
        if n == 0 {
            break;
        }
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

fn try_deser<T: serde::de::DeserializeOwned>(buf: &[u8]) -> Result<(T, usize), bincode::Error> {
    let mut cur = std::io::Cursor::new(buf);

    match bincode::deserialize_from(&mut cur) {
        Ok(m) => Ok((m, cur.position() as usize)),

        Err(e) => {
            match *e {
                // Incomplete input is okay; return the error as-is
                bincode::ErrorKind::Io(ref io_err)
                    if io_err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    Err(e)
                }

                // Any other error means invalid data; panic!
                _ => panic!("Invalid data during deserialization: {:?}", e),
            }
        }
    }
}

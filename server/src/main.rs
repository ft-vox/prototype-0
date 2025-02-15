use messages::{ClientMessage, ServerMessage};
use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

struct Server {
    client_map: BTreeMap<u32, Arc<Mutex<Client>>>, // Store clients with Mutex for safe access
}

impl Server {
    fn new() -> Server {
        Server {
            client_map: BTreeMap::new(),
        }
    }

    async fn handle_message(
        &mut self,
        client: &Arc<Mutex<Client>>,
        message: ClientMessage,
        server: Arc<Mutex<Server>>,
    ) {
        let player_id = client.lock().await.player_id;
        match message {
            ClientMessage::Move { position } => {
                let message = ServerMessage::PlayerMove {
                    moved_player_id: player_id,
                    position,
                };
                for (&client_player_id, client) in self.client_map.iter() {
                    if client_player_id != player_id {
                        let mut client_guard = client.lock().await;
                        client_guard.send(message.clone(), server.clone()).await;
                    }
                }
            }
            _ => {
                // TODO: remove _
            }
        };
    }

    async fn add_client(&mut self, client: Arc<Mutex<Client>>) {
        let player_id = client.lock().await.player_id;
        self.client_map.insert(player_id, client);
    }

    async fn remove_client(&mut self, player_id: u32) {
        self.client_map.remove(&player_id);
    }
}

struct Client {
    player_id: u32,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    buffer: Arc<Mutex<VecDeque<ServerMessage>>>,
    is_sender_spawned: Arc<Mutex<bool>>,
}

impl Client {
    fn new(player_id: u32, writer: OwnedWriteHalf) -> Client {
        Client {
            player_id,
            writer: Arc::new(Mutex::new(writer)),
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            is_sender_spawned: Arc::new(Mutex::new(false)),
        }
    }

    async fn send(&mut self, message: ServerMessage, server: Arc<Mutex<Server>>) {
        {
            self.buffer.lock().await.push_back(message);
        }

        let mut sender_spawned_guard = self.is_sender_spawned.lock().await;
        if !*sender_spawned_guard {
            let buffer = self.buffer.clone();
            let is_sender_spawned = self.is_sender_spawned.clone();
            let writer = self.writer.clone();
            let player_id = self.player_id;
            *sender_spawned_guard = true;
            tokio::spawn(async move {
                while let Some(message) = buffer.lock().await.pop_front() {
                    let response_bytes = bincode::serialize(&message).unwrap();
                    writer
                        .lock()
                        .await
                        .write_all(&response_bytes)
                        .await
                        .unwrap_or_else(|_| {
                            let server = server.clone();
                            tokio::spawn(async move {
                                server.lock().await.remove_client(player_id).await;
                            });
                        });
                }
                *is_sender_spawned.lock().await = false;
            });
        }
    }
}

#[tokio::main]
async fn main() {
    let port = env::args().nth(1).unwrap_or_else(|| "4242".to_string());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("ft_vox server running on port {}...", port);

    let mut last_player_id = 0;

    let server = Arc::new(Mutex::new(Server::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let player_id = last_player_id;
        last_player_id += 1;

        let server_clone = Arc::clone(&server);
        tokio::spawn(async move {
            handle_client(socket, player_id, server_clone).await;
        });
    }
}

async fn handle_client(socket: tokio::net::TcpStream, player_id: u32, server: Arc<Mutex<Server>>) {
    let (mut reader, writer) = socket.into_split();
    let client = Arc::new(Mutex::new(Client::new(player_id, writer)));

    {
        client
            .lock()
            .await
            .send(
                ServerMessage::Init {
                    your_player_id: player_id,
                    your_position: messages::PlayerPosition::NotInWorld,
                },
                server.clone(),
            )
            .await;
    }

    println!("Player {} connected", player_id);

    {
        let mut server_guard = server.lock().await;
        server_guard.add_client(client.clone()).await;
    }

    let mut buffer = Vec::new();

    while let Ok(n) = reader.read_buf(&mut buffer).await {
        if n == 0 {
            break;
        }

        while let Ok((message, consumed)) = try_deserialize::<ClientMessage>(&buffer) {
            buffer.drain(..consumed);

            let mut server_guard = server.lock().await;
            server_guard
                .handle_message(&client, message, server.clone())
                .await;
        }
    }

    {
        let mut server_guard = server.lock().await;
        server_guard.remove_client(player_id).await;
    }

    println!("Player {} disconnected", player_id);
}

fn try_deserialize<T: serde::de::DeserializeOwned>(
    buffer: &[u8],
) -> Result<(T, usize), bincode::Error> {
    let mut cursor = std::io::Cursor::new(buffer);
    match bincode::deserialize_from(&mut cursor) {
        Ok(msg) => Ok((msg, cursor.position() as usize)),
        Err(_) => Err(bincode::ErrorKind::SizeLimit.into()),
    }
}

use messages::{ClientMessage, ServerMessage};
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let port = env::args().nth(1).unwrap_or_else(|| "4242".to_string());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    println!("ft_vox server running on port {}...", port);

    let mut last_player_id = 0;

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let player_id = last_player_id;
        last_player_id += 1;
        tokio::spawn(handle_client(socket, player_id));
    }
}

async fn handle_client(socket: tokio::net::TcpStream, player_id: u32) {
    let (mut reader, mut writer) = socket.into_split();
    let mut buffer = Vec::new();

    while let Ok(n) = reader.read_buf(&mut buffer).await {
        if n == 0 {
            break; // Connection closed
        }

        while let Ok((request, consumed)) = try_deserialize::<ClientMessage>(&buffer) {
            buffer.drain(..consumed); // Remove processed bytes

            let response = match request {
                ClientMessage::Move { position } => Some(ServerMessage::PlayerMove {
                    moved_player_id: player_id,
                    position,
                }),
                _ => None,
            };

            if let Some(response) = response {
                let response_bytes = bincode::serialize(&response).unwrap();
                writer.write_all(&response_bytes).await.unwrap();
            }
        }
    }
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

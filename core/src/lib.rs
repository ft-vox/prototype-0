use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use glam::Vec3;
use messages::{ClientMessage, PlayerPosition, ServerMessage};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedWriteHalf, TcpStream},
};
use wgpu::util::DeviceExt;

mod graphics;
pub mod player;
pub mod terrain_manager;
mod terrain_worker;
pub mod vertex;

use graphics::VoxGraphicsWrapper;
use player::Human;
use terrain_manager::TerrainManager;

pub const CACHE_DISTANCE: usize = 30;
pub const RENDER_DISTANCE: f32 = CACHE_DISTANCE as f32;
pub const FOG_COLOR_SRGB: [f32; 4] = [130.0 / 255.0, 173.0 / 255.0, 253.0 / 255.0, 1.0];
pub const FOV: f32 = 80.0;

pub fn get_coords(distance: f32) -> Vec<(i32, i32)> {
    let mut coords = Vec::new();
    let max_coord = distance.floor() as i32;
    let distance_squared = distance * distance;

    for x in -max_coord..=max_coord {
        for y in -max_coord..=max_coord {
            let dist_sq = (x * x + y * y) as f32;
            if dist_sq <= distance_squared {
                coords.push((x, y));
            }
        }
    }

    coords
}

struct Server {
    player_id: Arc<Mutex<Option<u32>>>,
    writer: Arc<tokio::sync::Mutex<OwnedWriteHalf>>,
    send_buffer: Arc<tokio::sync::Mutex<VecDeque<ClientMessage>>>,
    is_sender_spawned: Arc<tokio::sync::Mutex<bool>>,
    receive_buffer: Arc<Mutex<VecDeque<ServerMessage>>>,
    receiver_arc: Arc<()>, // stop receiver on destroy
}

impl Server {
    fn new(stream: TcpStream) -> Server {
        let (mut reader, writer) = stream.into_split();

        let result = Server {
            player_id: Arc::new(Mutex::new(None)),
            send_buffer: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
            is_sender_spawned: Arc::new(tokio::sync::Mutex::new(false)),
            writer: Arc::new(tokio::sync::Mutex::new(writer)),
            receive_buffer: Arc::new(Mutex::new(VecDeque::new())),
            receiver_arc: Arc::new(()),
        };

        let receiver_weak = Arc::downgrade(&result.receiver_arc);
        let receive_buffer = result.receive_buffer.clone();
        tokio::spawn(async move {
            let mut buffer = Vec::new();

            while let Ok(n) = reader.read_buf(&mut buffer).await {
                if n == 0 {
                    println!("Disconnected");
                    break;
                }

                while let Ok((message, consumed)) = try_deserialize::<ServerMessage>(&buffer) {
                    buffer.drain(..consumed);

                    receive_buffer.lock().unwrap().push_back(message);
                }

                if receiver_weak.upgrade().is_none() {
                    break;
                }
            }
        });

        result
    }

    fn send(&mut self, message: ClientMessage) {
        let buffer = self.send_buffer.clone();
        let is_sender_spawned = self.is_sender_spawned.clone();
        let writer = self.writer.clone();

        tokio::spawn(async move {
            {
                buffer.lock().await.push_back(message);
            }

            let mut sender_spawned_guard = is_sender_spawned.lock().await;
            if !*sender_spawned_guard {
                *sender_spawned_guard = true;
                let is_sender_spawned = is_sender_spawned.clone();
                tokio::spawn(async move {
                    while let Some(message) = buffer.lock().await.pop_front() {
                        let response_bytes = bincode::serialize(&message).unwrap();
                        writer
                            .lock()
                            .await
                            .write_all(&response_bytes)
                            .await
                            .unwrap();
                    }
                    *is_sender_spawned.lock().await = false;
                });
            }
        });
    }
}

pub struct Vox {
    server: Arc<Mutex<Server>>,
    vox_graphics_wrapper: VoxGraphicsWrapper,
    local_player: Human,
    is_paused: bool,
    terrain_manager: TerrainManager,
    target_fog_distance: f32,
    current_fog_distance: f32,
}

impl Vox {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        stream: TcpStream,
    ) -> Self {
        let vox_graphics_wrapper = VoxGraphicsWrapper::init(config, _adapter, device, queue);
        let eye_x = 0.0;
        let eye_y = -5.0;
        let eye_z = 120.0;
        let server = Arc::new(Mutex::new(Server::new(stream)));

        // Done
        Vox {
            vox_graphics_wrapper,
            local_player: Human::new(Vec3::new(eye_x, eye_y, eye_z)),
            is_paused: false,
            terrain_manager: TerrainManager::new(CACHE_DISTANCE, (eye_x, eye_y)),
            target_fog_distance: 0.0,
            current_fog_distance: 0.0,
            server,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_is_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }

    pub fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.vox_graphics_wrapper.resize(config, device, _queue);
    }

    pub fn tick(
        &mut self,
        delta_time: f32,
        move_direction: [f32; 3],
        move_speed: player::MoveSpeed,
        delta_horizontal_rotation: f32,
        delta_vertical_rotation: f32,
    ) {
        {
            let server_guard = self.server.lock().unwrap();
            let player_id = server_guard.player_id.clone();
            let mut buffer_guard = server_guard.receive_buffer.lock().unwrap();
            while let Some(message) = buffer_guard.pop_front() {
                match message {
                    ServerMessage::Init {
                        your_player_id,
                        your_position,
                    } => {
                        player_id.lock().unwrap().replace(your_player_id);
                        if let PlayerPosition::InWorld {
                            position: [x, y, z],
                            horizontal_rotation,
                            vertical_rotation,
                        } = your_position
                        {
                            self.local_player.position = Vec3::new(x, y, z);
                            self.local_player.horizontal_rotation = horizontal_rotation;
                            self.local_player.vertical_rotation = vertical_rotation;
                        }
                    }
                    _ => {
                        // TODO: remove _
                    }
                }
            }
        }

        if self.is_paused {
            return;
        }

        self.local_player.move_speed = move_speed;
        self.local_player.update(
            delta_time,
            move_direction,
            delta_horizontal_rotation,
            delta_vertical_rotation,
        );

        // if player is moved
        {
            let mut server_guard = self.server.lock().unwrap();
            server_guard.send(ClientMessage::Move {
                position: PlayerPosition::InWorld {
                    position: [
                        self.local_player.position.x,
                        self.local_player.position.y,
                        self.local_player.position.z,
                    ],
                    horizontal_rotation: self.local_player.horizontal_rotation,
                    vertical_rotation: self.local_player.vertical_rotation,
                },
            });
        }

        // smooth fog distance
        {
            self.target_fog_distance = self.terrain_manager.get_farthest_distance().max(6.0);

            let step = 3.5 * delta_time;
            if self.target_fog_distance != self.current_fog_distance {
                let direction = (self.target_fog_distance - self.current_fog_distance).signum();
                self.current_fog_distance += step * direction;
                if (self.current_fog_distance - self.target_fog_distance) * direction > 0.0 {
                    self.current_fog_distance = self.target_fog_distance;
                }
            }
        }
    }

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.terrain_manager.set_cache_distance(CACHE_DISTANCE);
        let eye_pos = self.local_player.get_eye_position();
        self.terrain_manager.set_eye((eye_pos.x, eye_pos.y));

        let buffers = self.terrain_manager.get_available(&mut |mesh| {
            (
                Arc::new(
                    mesh.opaque_buffers
                        .iter()
                        .map(|(vertices, indices)| {
                            (
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Vertex Buffer"),
                                    contents: bytemuck::cast_slice(vertices),
                                    usage: wgpu::BufferUsages::VERTEX,
                                }),
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Index Buffer"),
                                    contents: bytemuck::cast_slice(indices),
                                    usage: wgpu::BufferUsages::INDEX,
                                }),
                                indices.len() as u32,
                            )
                        })
                        .collect(),
                ),
                Arc::new(
                    mesh.translucent_buffers
                        .iter()
                        .map(|(vertices, indices)| {
                            (
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Vertex Buffer"),
                                    contents: bytemuck::cast_slice(vertices),
                                    usage: wgpu::BufferUsages::VERTEX,
                                }),
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Index Buffer"),
                                    contents: bytemuck::cast_slice(indices),
                                    usage: wgpu::BufferUsages::INDEX,
                                }),
                                indices.len() as u32,
                            )
                        })
                        .collect(),
                ),
            )
        });

        self.vox_graphics_wrapper.update(
            self.local_player.get_eye_position(),
            self.local_player.get_eye_direction(),
        );
        self.vox_graphics_wrapper
            .render(view, device, queue, self.current_fog_distance, buffers);
    }
}

async fn send_message(writer: &mut tokio::net::tcp::OwnedWriteHalf, message: ClientMessage) {
    let message_bytes = bincode::serialize(&message).unwrap();
    writer.write_all(&message_bytes).await.unwrap();
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

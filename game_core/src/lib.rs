use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use glam::Vec3;
use kira::{
    sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
    AudioManager, AudioManagerSettings, DefaultBackend,
};
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

/// 블록 좌표 리스트를 구하는 예시 함수
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

/// 클라이언트측 'Server' 구조체
/// ─ 원래 `TcpStream` 전체를 받아 read/write 했으나,
///   이제는 '쓰기'만 담당하도록 OwnedWriteHalf만 보관
struct Server {
    /// 플레이어 ID(옵션)
    player_id: Arc<Mutex<Option<u32>>>,

    /// 서버에 write하는 half
    writer: Arc<tokio::sync::Mutex<OwnedWriteHalf>>,

    /// 전송할 ClientMessage 버퍼
    send_buffer: Arc<tokio::sync::Mutex<VecDeque<ClientMessage>>>,

    /// 메시지 전송 태스크 중복 방지
    is_sender_spawned: Arc<tokio::sync::Mutex<bool>>,

    /// (주석) 원래는 서버에서 오는 read 메시지를 보관하던 큐
    receive_buffer: Arc<Mutex<VecDeque<ServerMessage>>>,

    /// (주석) drop 시 리시버를 멈추기 위한 Arc
    receiver_arc: Arc<()>,
}

impl Server {
    /// new: 이제 `stream: TcpStream` 대신 `write_half: OwnedWriteHalf`
    ///
    /// 기존 코드에서 read loop 부분은 제거(또는 주석처리)했습니다.
    fn new(/*stream: TcpStream*/ write_half: OwnedWriteHalf) -> Server {
        // ──── 원래 read loop 있던 부분 주석처리 ────
        // let (mut reader, writer) = stream.into_split();
        // tokio::spawn(async move {
        //     let mut buffer = Vec::new();
        //     while let Ok(n) = reader.read_buf(&mut buffer).await {
        //         // ...
        //     }
        // });

        // 이제는 writer만 보관
        let result = Server {
            player_id: Arc::new(Mutex::new(None)),
            send_buffer: Arc::new(tokio::sync::Mutex::new(VecDeque::new())),
            is_sender_spawned: Arc::new(tokio::sync::Mutex::new(false)),
            writer: Arc::new(tokio::sync::Mutex::new(write_half)),

            // read loop 관련 필드들(사용 안함이지만, 한 줄도 생략하지 않는다는 요청으로 유지)
            receive_buffer: Arc::new(Mutex::new(VecDeque::new())),
            receiver_arc: Arc::new(()),
        };

        // (주석) read loop는 제거되었으므로 여기서 아무것도 안 함

        result
    }

    /// 서버에 ClientMessage 전송
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

/// 게임 전체를 관리하는 Vox 구조체
pub struct Vox {
    /// 서버(쓰기 전용)
    server: Arc<Mutex<Server>>,

    /// 그래픽
    vox_graphics_wrapper: VoxGraphicsWrapper,

    /// 로컬 플레이어
    local_player: Human,

    /// 일시정지 여부
    is_paused: bool,

    /// 지형
    terrain_manager: TerrainManager,

    /// 포그 거리
    target_fog_distance: f32,
    current_fog_distance: f32,

    /// 오디오
    audio_manager: AudioManager<DefaultBackend>,
    bgm_handle: StaticSoundHandle,
}

impl Vox {
    /// 이제 `TcpStream` 대신 `OwnedWriteHalf`를 받도록 수정
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        /*stream: TcpStream*/ write_half: OwnedWriteHalf,
    ) -> Self {
        let vox_graphics_wrapper = VoxGraphicsWrapper::init(config, _adapter, device, queue);

        // Server::new(...)도 `write_half`만
        let server = Arc::new(Mutex::new(Server::new(write_half)));

        let eye_x = 0.0;
        let eye_y = -5.0;
        let eye_z = 120.0;

        let mut audio_manager =
            AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .expect("오디오 매니저 생성 실패");
        let bgm_data =
            StaticSoundData::from_file("game_core/assets/bgm.mp3").expect("BGM 파일 로드 실패");
        let bgm_settings = StaticSoundSettings::new().loop_region(..);
        let bgm_handle = audio_manager
            .play(bgm_data.with_settings(bgm_settings))
            .expect("BGM 재생 실패");

        Vox {
            vox_graphics_wrapper,
            local_player: Human::new(Vec3::new(eye_x, eye_y, eye_z)),
            is_paused: false,
            terrain_manager: TerrainManager::new(CACHE_DISTANCE, (eye_x, eye_y)),
            target_fog_distance: 0.0,
            current_fog_distance: 0.0,
            server,
            audio_manager,
            bgm_handle,
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

    /// 매 프레임 로직
    pub fn tick(
        &mut self,
        delta_time: f32,
        move_direction: [f32; 3],
        move_speed: player::MoveSpeed,
        delta_horizontal_rotation: f32,
        delta_vertical_rotation: f32,
    ) {
        // (주석) 여기서 원래 receive_buffer에서 서버메시지 꺼내 처리했지만,
        // 이제는 클라이언트 main에서 read_half로 받고 있으므로 제거(또는 주석)

        if self.is_paused {
            return;
        }

        // 로컬 플레이어 업데이트
        self.local_player.move_speed = move_speed;
        self.local_player.update(
            delta_time,
            move_direction,
            delta_horizontal_rotation,
            delta_vertical_rotation,
        );

        // 플레이어 위치 → 서버 전송
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

        // 포그 거리 조정
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

    /// 렌더링
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

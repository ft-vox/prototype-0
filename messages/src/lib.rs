use map_types::{Chunk, Cube};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Move {
        position: PlayerPosition,
    },
    WatchChunk {
        x: i32,
        y: i32,
    },
    UnwatchChunk {
        x: i32,
        y: i32,
    },
    DestroyBlock {
        chunk_x: i32,
        chunk_y: i32,
        block_x: u32,
        block_y: u32,
        block_z: u32,
    },
    PutBlock {
        chunk_x: i32,
        chunk_y: i32,
        block_x: u32,
        block_y: u32,
        block_z: u32,
        cube: Cube,
    },
}

/// **중요**: `#[derive(Debug)]` 추가하여, `{:?}` 출력 가능하도록 함
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    Init {
        your_player_id: u32,
        your_position: PlayerPosition,
    },
    PlayerMove {
        moved_player_id: u32,
        position: PlayerPosition,
    },
    Chunk {
        x: i32,
        y: i32,
        chunk: Box<Chunk>,
    },
    DestroyBlock {
        chunk_x: i32,
        chunk_y: i32,
        block_x: u32,
        block_y: u32,
        block_z: u32,
    },
    PutBlock {
        chunk_x: i32,
        chunk_y: i32,
        block_x: u32,
        block_y: u32,
        block_z: u32,
        cube: Cube,
    },
    PlayerAction {
        action: PlayerAction,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayerAction {
    DestroyBlock,
}

/// `#[derive(Debug)]`를 여기에도 붙이면, `{:?}`로 PlayerPosition 출력 가능
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayerPosition {
    NotInWorld,
    InWorld {
        position: [f32; 3],
        horizontal_rotation: f32,
        vertical_rotation: f32,
    },
}

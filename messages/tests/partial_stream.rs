use messages::{ServerMessage, PlayerAction, PlayerPosition};
use bincode;

fn try_deser<T: serde::de::DeserializeOwned>(buf: &[u8]) -> Result<(T, usize), bincode::Error> {
    let mut cur = std::io::Cursor::new(buf);
    match bincode::deserialize_from(&mut cur) {
        Ok(m) => Ok((m, cur.position() as usize)),
        Err(e) => match *e {
            bincode::ErrorKind::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::UnexpectedEof => Err(e),
            _ => panic!("Invalid data during deserialization: {:?}", e),
        },
    }
}

#[test]
fn parse_two_messages_from_partial_third() {
    // prepare three messages
    let msg1 = ServerMessage::Init { your_player_id: 1, your_position: PlayerPosition::NotInWorld };
    let msg2 = ServerMessage::PlayerAction { action: PlayerAction::DestroyBlock };
    let msg3 = ServerMessage::Init { your_player_id: 2, your_position: PlayerPosition::NotInWorld };

    let mut buf = Vec::new();
    bincode::serialize_into(&mut buf, &msg1).unwrap();
    bincode::serialize_into(&mut buf, &msg2).unwrap();
    let partial = bincode::serialize(&msg3).unwrap();
    let cut = partial.len() / 2;
    buf.extend_from_slice(&partial[..cut]);

    // first message
    let (parsed1, consumed1) = try_deser::<ServerMessage>(&buf).unwrap();
    match parsed1 {
        ServerMessage::Init { your_player_id, .. } => assert_eq!(your_player_id, 1),
        _ => panic!("Unexpected variant for first message"),
    }
    buf.drain(..consumed1);

    // second message
    let (parsed2, consumed2) = try_deser::<ServerMessage>(&buf).unwrap();
    match parsed2 {
        ServerMessage::PlayerAction { action: PlayerAction::DestroyBlock } => {},
        _ => panic!("Unexpected variant for second message"),
    }
    buf.drain(..consumed2);

    // third message should be incomplete
    assert!(try_deser::<ServerMessage>(&buf).is_err());
}

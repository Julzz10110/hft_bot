use log::{debug, error};
use std::io::{Cursor, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

#[derive(Debug, Clone)]
pub struct BinaryMessage {
    pub message_type: u8,
    pub data: Vec<u8>,
}

impl BinaryMessage {
    pub fn new(message_type: u8, data: Vec<u8>) -> Self {
        BinaryMessage {
            message_type,
            data,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer: Vec<u8> = Vec::new();

        // write message type
        buffer.write_u8(self.message_type)?;

        // write data length (as u32)
        buffer.write_u32::<LittleEndian>(self.data.len() as u32)?;

        // write data
        buffer.write_all(&self.data)?;

        Ok(buffer)
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(buffer);

        // read message type
        let message_type = cursor.read_u8()?;

        // read data length
        let data_length = cursor.read_u32::<LittleEndian>()? as usize;

        // read data
        let mut data: Vec<u8> = vec![0; data_length];
        cursor.read_exact(&mut data)?;

        Ok(BinaryMessage {
            message_type,
            data,
        })
    }
}

pub fn parse_binary_message(data: &[u8]) -> Result<BinaryMessage, Box<dyn std::error::Error>> {
    debug!("Parsing binary message");
    let message = BinaryMessage::deserialize(data)?;
    Ok(message)
}

pub fn format_binary_message(message_type: u8, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    debug!("Formatting binary message");
    let binary_message = BinaryMessage::new(message_type, data.to_vec());
    binary_message.serialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_message_serialization_deserialization() {
        let original_message = BinaryMessage::new(1, vec![1, 2, 3, 4, 5]);
        let serialized_message = original_message.serialize().unwrap();
        let deserialized_message = BinaryMessage::deserialize(&serialized_message).unwrap();

        assert_eq!(original_message.message_type, deserialized_message.message_type);
        assert_eq!(original_message.data, deserialized_message.data);
    }
}


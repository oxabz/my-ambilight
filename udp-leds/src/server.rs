/**
 * # Server Messages
 * Defines the messages that the server can send to the client.
 * 
 * ## Message format
 * The messages are sent as a byte array
 * The first byte is the flag
 * The second byte is the instruction
 * The instruction is the 2 most significant bits of the second byte
 * 
 * ## Hello
 * The server sends a hello message to the client to confirm that it is the server
 * [SERVER_FLAG, 0b1100_0000]
 */
#[derive(Debug , PartialEq)]
pub enum ServerMessages {
    Hello
}

impl ServerMessages {
    /// Creates a new hello message
    pub fn hello() -> Self {
        ServerMessages::Hello
    }
}

impl TryFrom<&[u8]> for ServerMessages {
    type Error = crate::error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() <= 2 {
            return Err(crate::error::Error::InvalidMessageLength);
        }
        if value[0] != crate::constants::SERVER_FLAG {
            return Err(crate::error::Error::InvalidFlag);
        }

        match value[1] & crate::constants::INSTRUCTION_MASK {
            crate::constants::INSTRUCTION_HELLO => Ok(ServerMessages::Hello),
            _ => Err(crate::error::Error::InvalidFlag)
        }
    }
}

impl Into<[u8; crate::constants::MAX_MESSAGE_LENGTH]> for ServerMessages  {
    fn into(self) -> [u8; crate::constants::MAX_MESSAGE_LENGTH] {
        match self {
            ServerMessages::Hello => {
                let mut message = [0; crate::constants::MAX_MESSAGE_LENGTH];
                message[0] = crate::constants::SERVER_FLAG;
                message[1] = crate::constants::INSTRUCTION_HELLO;
                message
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let message: [u8; 770] = ServerMessages::Hello.into();
        assert!(message[0] == crate::constants::SERVER_FLAG);
        assert!(message[1] == crate::constants::INSTRUCTION_HELLO);
        let parsed = ServerMessages::try_from(&message[..]).unwrap();
        assert_eq!(parsed, ServerMessages::Hello);
    }

    #[test]
    fn test_invalid_flag() {
        let mut message: [u8; 770] = ServerMessages::Hello.into();
        message[0] = 0;
        let parsed = ServerMessages::try_from(&message[..]);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_length() {
        let mut message: [u8; 770] = ServerMessages::Hello.into();
        message[1] = 0;
        let parsed = ServerMessages::try_from(&message[..]);
        assert!(parsed.is_err());
    }
}
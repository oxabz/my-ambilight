use crate::constants::{CLIENT_FLAG, INSTRUCTION_MASK, MAX_MESSAGE_LENGTH, MAX_LED_COUNT};

#[derive(Debug, Clone)]
pub enum ClientMessages {
    Hello,
    SetActive(u8),
    SendPixels([u8; MAX_LED_COUNT * 3]),
    SetPixel(u8, u8, u8, u8)
}

impl TryFrom<&[u8]> for ClientMessages {
    type Error = crate::error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 2 || value.len() > MAX_MESSAGE_LENGTH {
            return Err(crate::error::Error::InvalidMessageLength);
        }
        if value[0] != CLIENT_FLAG {
            return Err(crate::error::Error::InvalidFlag);
        }

        match value[1] & INSTRUCTION_MASK {
            crate::constants::INSTRUCTION_HELLO => Ok(ClientMessages::Hello),
            crate::constants::INSTRUCTION_SET_ACTIVE => {
                if value.len() != 2 {
                    return Err(crate::error::Error::InvalidMessageLength);
                }
                Ok(ClientMessages::SetActive(value[1] & crate::constants::DEVICE_MASK))
            },
            crate::constants::INSTRUCTION_SEND_PIXELS => {
                let count = value.len() - 2;
                let mut pixels = [0; MAX_LED_COUNT * 3];
                pixels[..count].copy_from_slice(&value[2..MAX_LED_COUNT * 3 + 2]);
                Ok(ClientMessages::SendPixels(pixels))
            },
            crate::constants::INSTRUCTION_SET_PIXEL => {
                if value.len() != 5 {
                    return Err(crate::error::Error::InvalidMessageLength);
                }
                Ok(ClientMessages::SetPixel(value[1] & crate::constants::DEVICE_MASK, value[2], value[3], value[4]))
            },
            _ => panic!("Unreachable")
            
        }
    }
}

impl Into<[u8; MAX_MESSAGE_LENGTH]> for ClientMessages  {
    fn into(self) -> [u8; MAX_MESSAGE_LENGTH] {
        match self {
            ClientMessages::Hello => {
                let mut message = [0; MAX_MESSAGE_LENGTH];
                message[0] = CLIENT_FLAG;
                message[1] = crate::constants::INSTRUCTION_HELLO;
                message
            },
            ClientMessages::SetActive(device) => {
                let mut message = [0; MAX_MESSAGE_LENGTH];
                message[0] = CLIENT_FLAG;
                message[1] = crate::constants::INSTRUCTION_SET_ACTIVE | device;
                message
            },
            ClientMessages::SendPixels(pixels) => {
                let mut message = [0; MAX_MESSAGE_LENGTH];
                message[0] = CLIENT_FLAG;
                message[1] = crate::constants::INSTRUCTION_SEND_PIXELS;
                message[2..pixels.len() + 2].copy_from_slice(&pixels);
                message
            },
            ClientMessages::SetPixel(device, r, g, b) => {
                let mut message = [0; MAX_MESSAGE_LENGTH];
                message[0] = CLIENT_FLAG;
                message[1] = crate::constants::INSTRUCTION_SET_PIXEL | device;
                message[2] = r;
                message[3] = g;
                message[4] = b;
                message
            }
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_hello() {
        let message = ClientMessages::Hello;
        let bytes: [u8; MAX_MESSAGE_LENGTH] = message.into();
        assert_eq!(bytes[0], CLIENT_FLAG);
        assert_eq!(bytes[1], crate::constants::INSTRUCTION_HELLO);
    }

    #[test]
    fn test_set_active() {
        let message = ClientMessages::SetActive(1);
        let bytes: [u8; MAX_MESSAGE_LENGTH] = message.into();
        assert_eq!(bytes[0], CLIENT_FLAG);
        assert_eq!(bytes[1], crate::constants::INSTRUCTION_SET_ACTIVE | 1);
    }

    #[test]
    fn test_send_pixels() {
        let message = ClientMessages::SendPixels([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let bytes: [u8; MAX_MESSAGE_LENGTH] = message.into();
        assert_eq!(bytes[0], CLIENT_FLAG);
        assert_eq!(bytes[1], crate::constants::INSTRUCTION_SEND_PIXELS);
        assert_eq!(bytes[2], 1);
        assert_eq!(bytes[3], 2);
        assert_eq!(bytes[4], 3);
        assert_eq!(bytes[5], 4);
        assert_eq!(bytes[6], 5);
        assert_eq!(bytes[7], 6);
        assert_eq!(bytes[8], 7);
        assert_eq!(bytes[9], 8);
        assert_eq!(bytes[10], 9);
        assert_eq!(bytes[11], 10);
        assert_eq!(bytes[12], 11);
        assert_eq!(bytes[13], 12);
    }

    #[test]
    fn test_set_pixel() {
        let message = ClientMessages::SetPixel(1, 2, 3, 4);
        let bytes: [u8; MAX_MESSAGE_LENGTH] = message.into();
        assert_eq!(bytes[0], CLIENT_FLAG);
        assert_eq!(bytes[1], crate::constants::INSTRUCTION_SET_PIXEL | 1);
        assert_eq!(bytes[2], 2);
        assert_eq!(bytes[3], 3);
        assert_eq!(bytes[4], 4);
    }

    #[test]
    fn test_try_from_hello() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_HELLO];
        let message = ClientMessages::try_from(&bytes).unwrap();
        assert_eq!(message, ClientMessages::Hello);
    }

    #[test]
    fn test_try_from_set_active() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SET_ACTIVE | 1];
        let message = ClientMessages::try_from(&bytes).unwrap();
        assert_eq!(message, ClientMessages::SetActive(1));
    }

    #[test]
    fn test_try_from_send_pixels() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SEND_PIXELS, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let message = ClientMessages::try_from(&bytes).unwrap();
        assert_eq!(message, ClientMessages::SendPixels([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]));
    }

    #[test]
    fn test_try_from_set_pixel() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SET_PIXEL | 1, 2, 3, 4];
        let message = ClientMessages::try_from(&bytes).unwrap();
        assert_eq!(message, ClientMessages::SetPixel(1, 2, 3, 4));
    }

    #[test]
    fn test_try_from_invalid_flag() {
        let bytes = [0x00, crate::constants::INSTRUCTION_HELLO];
        let message = ClientMessages::try_from(&bytes);
        assert_eq!(message, Err(crate::error::Error::InvalidFlag));
    }

    #[test]
    fn test_try_from_invalid_message_length() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_HELLO, 1];
        let message = ClientMessages::try_from(&bytes);
        assert_eq!(message, Err(crate::error::Error::InvalidMessageLength));
    }

    #[test]
    fn test_try_from_invalid_message_length_2() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SET_ACTIVE | 1, 1];
        let message = ClientMessages::try_from(&bytes);
        assert_eq!(message, Err(crate::error::Error::InvalidMessageLength));
    }
}
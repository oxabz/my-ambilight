use crate::{constants::{CLIENT_FLAG, INSTRUCTION_MASK, MAX_MESSAGE_LENGTH, MAX_LED_COUNT, DEVICE_MASK}, server::ServerMessages};

/**
 * # Client messages
 * Defines the messages that can be sent from the client to the server
 * 
 * ## Message format
 * The messages are sent as a byte array
 * The first byte is the flag
 * The second byte is the instruction and the device number which identifies the client
 * The devic number is the 6 least significant bits of the second byte giving a maximum of 64 devices
 * 
 * ## Hello
 * The client broadcasts a hello message to find the server
 * [CLIENT_FLAG, 0b1100_0000]
 * 
 * ## SetActive
 * The client sends a set active message to set the active device to the given device
 * Only one device can be active at a time
 * Only the active device will be able to update the LEDs
 * [CLIENT_FLAG, 0b0100_0000 | device]
 * 
 * ## SendPixels
 * The client sends a send pixels message to update the LEDs
 * The message contains a list of 24bits RGB values
 * The modifications are only applied if the current device is active
 * [CLIENT_FLAG, 0b0000_0000 | device, r1, g1, b1, r2, g2, b2, ...]
 * 
 * ## SetPixel
 * The client sends a set pixel message to update a single pixel
 * The message contains the index of the pixel and the 24bits RGB value
 * The modifications are only applied if the current device is active
 * [CLIENT_FLAG, 0b1000_0000 | device, index, r, g, b]
 */
#[derive(Debug, Clone, PartialEq)]
pub enum ClientMessages {
    Hello,
    SetActive(u8),
    SendPixels(u8, [u8; MAX_LED_COUNT * 3]),
    SetPixel(u8, u8, u8, u8, u8)
}

impl ClientMessages {
    /// Creates a new hello message
    pub fn hello() -> Self {
        ClientMessages::Hello
    }

    /// Creates a new set active message
    pub fn set_active(device: u8) -> Self {
        assert!(device < DEVICE_MASK, "Invalid device number: {}", device);
        ClientMessages::SetActive(device)
    }

    /// Creates a new send pixels message
    pub fn send_pixels(device: u8, pixels: [u8; MAX_LED_COUNT * 3]) -> Self {
        assert!(device < DEVICE_MASK, "Invalid device number: {}", device);
        ClientMessages::SendPixels(device, pixels)
    }

    /// Creates a new set pixel message
    pub fn set_pixel(device: u8, pixel:u8, r: u8, g: u8, b: u8) -> Self {
        assert!(device < DEVICE_MASK, "Invalid device number: {}", device);
        ClientMessages::SetPixel(device, pixel, r, g, b)
    }

    pub fn expect_response(&self) -> bool {
        match self {
            ClientMessages::Hello => true,
            ClientMessages::SetActive(_) => false,
            ClientMessages::SendPixels(_, _) => false,
            ClientMessages::SetPixel(_, _, _, _, _) => false
        }
    }

    pub fn response(&self) -> Option<ServerMessages> {
        match self {
            ClientMessages::Hello => Some(ServerMessages::Hello),
            ClientMessages::SetActive(_) => None,
            ClientMessages::SendPixels(_, _) => None,
            ClientMessages::SetPixel(_, _, _, _, _) => None
        }
    }
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
                if value.len() < 2 {
                    return Err(crate::error::Error::InvalidMessageLength);
                }
                Ok(ClientMessages::SetActive(value[1] & crate::constants::DEVICE_MASK))
            },
            crate::constants::INSTRUCTION_SEND_PIXELS => {
                let count = value.len() - 2;
                let mut pixels = [0; MAX_LED_COUNT * 3];
                pixels[..count].copy_from_slice(&value[2..MAX_LED_COUNT * 3 + 2]);
                Ok(ClientMessages::SendPixels(value[1] & crate::constants::DEVICE_MASK, pixels))
            },
            crate::constants::INSTRUCTION_SET_PIXEL => {
                if value.len() < 6 {
                    return Err(crate::error::Error::InvalidMessageLength);
                }
                Ok(ClientMessages::SetPixel(value[1] & crate::constants::DEVICE_MASK, value[2], value[3], value[4], value[5]))
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
            ClientMessages::SendPixels(device, pixels) => {
                let mut message = [0; MAX_MESSAGE_LENGTH];
                message[0] = CLIENT_FLAG;
                message[1] = crate::constants::INSTRUCTION_SEND_PIXELS | device;
                message[2..pixels.len() + 2].copy_from_slice(&pixels);
                message
            },
            ClientMessages::SetPixel(device, pixel, r, g, b) => {
                let mut message = [0; MAX_MESSAGE_LENGTH];
                message[0] = CLIENT_FLAG;
                message[1] = crate::constants::INSTRUCTION_SET_PIXEL | device;
                message[2] = pixel;
                message[3] = r;
                message[4] = g;
                message[5] = b;
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
        let mut bytes = [0; MAX_LED_COUNT * 3];
        bytes[0..13].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let message = ClientMessages::SendPixels(0, bytes);
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
        let message = ClientMessages::SetPixel(1, 2, 3, 4, 5);
        let bytes: [u8; MAX_MESSAGE_LENGTH] = message.into();
        assert_eq!(bytes[0], CLIENT_FLAG);
        assert_eq!(bytes[1], crate::constants::INSTRUCTION_SET_PIXEL | 1);
        assert_eq!(bytes[2], 2);
        assert_eq!(bytes[3], 3);
        assert_eq!(bytes[4], 4);
        assert_eq!(bytes[5], 5);
    }

    #[test]
    fn test_try_from_hello() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_HELLO];
        let message = ClientMessages::try_from(&bytes[..]).unwrap();
        assert_eq!(message, ClientMessages::Hello);
    }

    #[test]
    fn test_try_from_set_active() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SET_ACTIVE | 1];
        let message = ClientMessages::try_from(&bytes[..]).unwrap();
        assert_eq!(message, ClientMessages::SetActive(1));
    }

    #[test]
    fn test_try_from_send_pixels() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SEND_PIXELS, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let message = ClientMessages::try_from(&bytes[..]).unwrap();

        let mut bytes = [0; MAX_LED_COUNT * 3];
        bytes[0..13].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        assert_eq!(message, ClientMessages::SendPixels(0,bytes));
    }

    #[test]
    fn test_try_from_set_pixel() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SET_PIXEL | 1, 0, 2, 3, 4];
        
        let message = ClientMessages::try_from(&bytes[..]).unwrap();
        assert_eq!(message, ClientMessages::SetPixel(1, 0, 2, 3, 4));
    }

    #[test]
    fn test_try_from_invalid_flag() {
        let bytes = [0x00, crate::constants::INSTRUCTION_HELLO];
        let message = ClientMessages::try_from(&bytes[..]);
        assert_eq!(message, Err(crate::error::Error::InvalidFlag));
    }

    #[test]
    fn test_try_from_invalid_message_length() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_HELLO, 1];
        let message = ClientMessages::try_from(&bytes[..]);
        assert_eq!(message, Err(crate::error::Error::InvalidMessageLength));
    }

    #[test]
    fn test_try_from_invalid_message_length_2() {
        let bytes = [CLIENT_FLAG, crate::constants::INSTRUCTION_SET_ACTIVE | 1, 1];
        let message = ClientMessages::try_from(&bytes[..]);
        assert_eq!(message, Err(crate::error::Error::InvalidMessageLength));
    }

}
pub const MAX_LED_COUNT: usize = 256;
pub const MAX_MESSAGE_LENGTH: usize = MAX_LED_COUNT * 3 + 2;
pub const PORT: u16 = 52772;

pub(crate) const SERVER_FLAG: u8 = 0b1110_0110;
pub(crate) const CLIENT_FLAG: u8 = 0b0110_1011;

pub(crate) const INSTRUCTION_MASK: u8 = 0b1100_0000;
pub(crate) const DEVICE_MASK: u8 = 0b0011_1111;

pub(crate) const INSTRUCTION_HELLO: u8 = 0b1100_0000;
pub(crate) const INSTRUCTION_SET_ACTIVE: u8 = 0b0100_0000;
pub(crate) const INSTRUCTION_SEND_PIXELS: u8 = 0b0000_0000;
pub(crate) const INSTRUCTION_SET_PIXEL: u8 = 0b1000_0000;

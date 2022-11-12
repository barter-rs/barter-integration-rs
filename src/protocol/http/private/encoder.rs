/// Encodes bytes data.
pub trait Encoder {
    /// Encodes the bytes data into some `String` format.
    fn encode<Bytes>(&self, data: Bytes) -> String
    where
        Bytes: AsRef<[u8]>;
}

/// Encodes bytes data as a hex `String` using lowercase characters.
#[derive(Debug, Copy, Clone)]
pub struct HexEncoder;

impl Encoder for HexEncoder {
    fn encode<Bytes>(&self, data: Bytes) -> String
    where
        Bytes: AsRef<[u8]>,
    {
        hex::encode(data)
    }
}

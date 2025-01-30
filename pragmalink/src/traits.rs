pub trait AsHex {
    fn as_hex_string(&self) -> String;
}

pub trait AsBytes {
    fn as_bytes(&self) -> Vec<u8>;
}

impl<T> AsHex for T
where
    T: AsBytes,
{
    fn as_hex_string(&self) -> String {
        hex::encode(self.as_bytes())
    }
}

impl AsBytes for &[u8] {
    fn as_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

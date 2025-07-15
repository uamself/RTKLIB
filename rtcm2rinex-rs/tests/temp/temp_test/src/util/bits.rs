pub struct BitReader {
    pub data: Vec<u8>,
}

impl BitReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

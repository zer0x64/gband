#[derive(Default, Clone)]
pub struct OamDma {
    pub cycle: Option<u8>,
    pub source: u8,
}

impl OamDma {
    pub fn new(source: u8) -> Self {
        Self {
            source,
            cycle: Some(0),
        }
    }
}

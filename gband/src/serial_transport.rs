pub trait SerialTransport: Sync + Send {
    fn connect(&mut self) -> bool;

    fn is_connected(&self) -> bool;

    fn reset(&mut self);

    fn send(&mut self, data: u8);

    fn recv(&mut self) -> Option<u8>;
}

pub struct NullSerialTransport;

impl SerialTransport for NullSerialTransport {
    fn connect(&mut self) -> bool {
        false
    }

    fn is_connected(&self) -> bool {
        false
    }

    fn reset(&mut self) {}

    fn send(&mut self, _data: u8) {}

    fn recv(&mut self) -> Option<u8> {
        None
    }
}

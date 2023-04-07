use std::net::TcpStream;

pub type CommandProcessorType = Box<dyn CommandProcessor + Send>;

pub trait CommandProcessor {
    fn accept(&self, root_command: &str) -> bool;
    fn process(&self, full_command: &str, stream: &mut TcpStream) -> bool;
}

pub type CommandProcessorType = Box<dyn CommandProcessor>;

pub trait CommandProcessor {
    fn accept(&mut self, root_command: &str) -> bool;
    fn process(&self, full_command: &str) -> bool;
}

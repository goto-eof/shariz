use crate::structures::command_processor::CommandProcessor;

pub struct PullProcessor {}

impl CommandProcessor for PullProcessor {
    fn accept(&mut self, root_command: &str) -> bool {
        todo!()
    }

    fn process(&self, full_command: &str) -> bool {
        todo!()
    }
}

impl PullProcessor {}

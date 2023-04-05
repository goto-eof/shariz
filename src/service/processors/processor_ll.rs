use crate::structures::command_processor::CommandProcessor;

pub struct LLProcessor {}

impl CommandProcessor for LLProcessor {
    fn accept(&mut self, root_command: &str) -> bool {
        todo!()
    }

    fn process(&self, full_command: &str) -> bool {
        todo!()
    }
}

impl LLProcessor {}

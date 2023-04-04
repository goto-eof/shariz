use std::{
    io::{Stdout, Write},
    sync::{Arc, RwLock},
};

use crossterm::{
    cursor, queue,
    style::{self, Stylize},
    terminal, QueueableCommand,
};

pub async fn clear_console(stdout_rw_lock: Arc<RwLock<Stdout>>) -> () {
    let mut stdout = stdout_rw_lock.write().unwrap();
    stdout
        .queue(terminal::Clear(terminal::ClearType::All))
        .unwrap();
}

pub async fn print_message(stdout_rw_lock: Arc<RwLock<Stdout>>, line: u16, message: &str) -> () {
    let mut stdout = stdout_rw_lock.write().unwrap();

    let result = queue!(
        stdout,
        cursor::MoveTo(0, line),
        terminal::Clear(terminal::ClearType::UntilNewLine),
        style::PrintStyledContent(message.green())
    );
    if result.is_err() {
        // do nothing
    }
    let result = stdout.flush();
    if result.is_err() {
        // do nothing
    }
}

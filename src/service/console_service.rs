use std::io::{Stdout, Write};

use crossterm::{
    cursor, queue,
    style::{self, Stylize},
    terminal, QueueableCommand,
};

pub async fn clear_console(stdout: &mut Stdout) -> () {
    stdout
        .queue(terminal::Clear(terminal::ClearType::All))
        .unwrap();
}

pub async fn print_message(stdout: &mut Stdout, line: u16, message: &str) -> () {
    let result = queue!(
        stdout,
        cursor::MoveTo(0, line),
        terminal::Clear(terminal::ClearType::UntilNewLine),
        style::PrintStyledContent(message.green()),
        cursor::MoveTo(0, 0),
    );
    if result.is_err() {
        // do nothing
    }
    let result = stdout.flush();
    if result.is_err() {
        // do nothing
    }
}

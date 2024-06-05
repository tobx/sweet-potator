use std::fmt;

use crate::APP_NAME;

use super::color::Colorize;

#[derive(Clone, Copy)]
enum NotificationType {
    Error,
    Info,
    Success,
}

pub fn error<D: fmt::Display>(message: D) -> String {
    notify(NotificationType::Error, message)
}

pub fn info<D: fmt::Display>(message: D) -> String {
    notify(NotificationType::Info, message)
}

pub fn success<D: fmt::Display>(message: D) -> String {
    notify(NotificationType::Success, message)
}

fn notify<D: fmt::Display>(notification_type: NotificationType, message: D) -> String {
    let type_text = match notification_type {
        NotificationType::Error => "ERROR".red(),
        NotificationType::Info => "INFO".yellow(),
        NotificationType::Success => "SUCCESS".green(),
    };
    format!("{APP_NAME} ({type_text}): {message}")
}

pub mod write {
    use std::{fmt, io};

    use crate::terminal::{ewriteln, writeln};

    pub fn error<D: fmt::Display>(message: D) -> io::Result<()> {
        ewriteln(super::error(message))
    }

    pub fn info<D: fmt::Display>(message: D) -> io::Result<()> {
        writeln(super::info(message))
    }

    pub fn success<D: fmt::Display>(message: D) -> io::Result<()> {
        writeln(super::success(message))
    }
}

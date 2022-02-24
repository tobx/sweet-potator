use std::{
    io,
    path::Path,
    process::{Command, ExitStatus},
};

use crate::{
    error::{Error, Result},
    terminal::{color::Colorize, message::write},
};

pub fn open(command_with_args: &[String], path: &Path) -> Result<ExitStatus> {
    let (command, args) = command_with_args
        .split_first()
        .map_or(("", [].as_ref()), |(command, args)| (command, args));

    write::info("waiting for editor to close...")?;
    Command::new(command)
        .args(args)
        .arg(path)
        .status()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                Error::EditorCommandNotFound(command.yellow())
            } else {
                error.into()
            }
        })
}

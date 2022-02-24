pub mod color;
pub mod message;

use std::io::{self, BufWriter, Write};

pub fn write<S: AsRef<str>>(message: S) -> io::Result<()> {
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    out.write_all(message.as_ref().as_bytes())?;
    out.flush()
}

pub fn writeln<S: AsRef<str>>(message: S) -> io::Result<()> {
    write(message)?;
    write("\n")
}

pub fn ewrite<S: AsRef<str>>(message: S) -> io::Result<()> {
    let out = io::stderr();
    let mut out = BufWriter::new(out.lock());
    out.write_all(message.as_ref().as_bytes())?;
    out.flush()
}

pub fn ewriteln<S: AsRef<str>>(message: S) -> io::Result<()> {
    ewrite(message)?;
    ewrite("\n")
}

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};

const DEFAULT_BUF_SIZE: usize = 8192;

pub fn buffer_size() -> usize {
    DEFAULT_BUF_SIZE
}

pub struct BufInput<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> BufInput<R> {
    pub fn new(inner: R) -> Self {
        BufInput {
            reader: BufReader::with_capacity(buffer_size(), inner),
        }
    }

    pub fn with_capacity(cap: usize, inner: R) -> Self {
        BufInput {
            reader: BufReader::with_capacity(cap, inner),
        }
    }

    pub fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.reader.read_line(buf)
    }

    pub fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.reader.read_until(byte, buf)
    }
}

impl<R: Read> Read for BufInput<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

pub struct BufOutput<W: Write> {
    writer: BufWriter<W>,
    line_buffered: bool,
}

impl<W: Write> BufOutput<W> {
    pub fn new(inner: W) -> Self {
        BufOutput {
            writer: BufWriter::with_capacity(buffer_size(), inner),
            line_buffered: false,
        }
    }

    pub fn line_buffered(inner: W) -> Self {
        BufOutput {
            writer: BufWriter::with_capacity(buffer_size(), inner),
            line_buffered: true,
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> Write for BufOutput<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(buf)?;
        if self.line_buffered && buf.contains(&b'\n') {
            self.writer.flush()?;
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

pub fn is_terminal(fd: i32) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}

pub fn stdin_is_terminal() -> bool {
    is_terminal(0)
}

pub fn stdout_is_terminal() -> bool {
    is_terminal(1)
}

pub fn stderr_is_terminal() -> bool {
    is_terminal(2)
}

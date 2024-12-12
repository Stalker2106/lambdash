use std::io::{self, Write};

struct VirtualOutput {
    buffer: Vec<u8>,
}

impl VirtualOutput {
    fn new() -> Self {
        VirtualOutput { buffer: Vec::new() }
    }

    fn pipe(&mut self, out: &mut dyn Write) {
        let mut handle = out.lock();
        handle.write_all(&self.buffer);
    }
}

impl Write for VirtualOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return self.buffer.write(buf);
    }

    fn flush(&mut self) -> io::Result<()> {
        return self.buffer.flush();
    }
}

struct CommandOutput {
    stdout: VirtualOutput,
    stderr: VirtualOutput,
    status: Option<u8>
}



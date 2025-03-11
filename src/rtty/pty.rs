use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

type PtyReader = Box<dyn Read + Send>;
type PtyWriter = Box<dyn Write + Send>;

pub struct Pty {
    pub reader: Arc<Mutex<PtyReader>>,
    pub writer: Arc<Mutex<PtyWriter>>,
}

impl Pty {
    pub fn new(platform: &Platform) -> Result<Self, Error> {
        let pty = NativePtySystem::default()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_height: 0,
                pixel_width: 0,
            })
            .handle_err(location!())?;

        let _ = pty
            .slave
            .spawn_command(CommandBuilder::new(pty_program(platform)))
            .handle_err(location!())?;

        let reader = pty.master.try_clone_reader().handle_err(location!())?;
        let writer = pty.master.take_writer().handle_err(location!())?;

        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
        })
    }
}

impl Write for Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer
            .lock()
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to acquire writer lock: write operation aborted",
                )
            })?
            .write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer
            .lock()
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to acquire writer lock: flush operation aborted",
                )
            })?
            .flush()
    }
}

impl Read for Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader
            .lock()
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to acquire reader lock: read operation aborted",
                )
            })?
            .read(buf)
    }
}

fn pty_program(platform: &Platform) -> &'static str {
    match platform {
        Platform::PfSense => "/etc/rc.initial",
        Platform::OPNsense => "/bin/sh",
    }
}

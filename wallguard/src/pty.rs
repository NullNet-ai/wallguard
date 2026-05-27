use nullnet_liberror::{Error, ErrorHandler, Location, location};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::sync::Mutex;
use std::{
    io::{Read, Write},
    sync::Arc,
};

pub type PtyReader = Arc<Mutex<Box<dyn Read + Send>>>;
pub type PtyWriter = Arc<Mutex<Box<dyn Write + Send>>>;

#[derive(Clone)]
pub struct Pty {
    pub reader: PtyReader,
    pub writer: PtyWriter,
}

fn default_shell() -> &'static str {
    #[cfg(windows)]
    {
        "powershell.exe"
    }
    #[cfg(not(windows))]
    {
        "/bin/sh"
    }
}

impl Pty {
    /// Opens a PTY running the OS-appropriate shell:
    /// `powershell.exe` on Windows, `/bin/sh` everywhere else.
    pub fn new_shell() -> Result<Self, Error> {
        Self::new(default_shell())
    }

    pub fn new(command: &str) -> Result<Self, Error> {
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
            .spawn_command(CommandBuilder::new(command))
            .handle_err(location!())?;

        let writer = pty.master.take_writer().handle_err(location!())?;
        let reader = pty.master.try_clone_reader().handle_err(location!())?;

        Ok(Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
        })
    }
}

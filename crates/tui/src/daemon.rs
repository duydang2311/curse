use std::{io::Write, path::PathBuf};

use common::{get_socket_name, msg::ClientMsg};
use interprocess::local_socket::{Stream, prelude::*};

#[derive(Debug)]
pub struct DaemonClient {
    stream: Option<Stream>,
}

impl DaemonClient {
    pub fn new() -> DaemonClient {
        DaemonClient { stream: None }
    }

    fn get_stream(&mut self) -> eyre::Result<&mut Stream> {
        if self.stream.is_none() {
            let name = get_socket_name()?;
            self.stream = Some(Stream::connect(name)?);
        }
        Ok(self.stream.as_mut().expect("unreachable"))
    }

    pub fn play(&mut self, path: PathBuf) -> eyre::Result<()> {
        self.write_all(&postcard::to_allocvec(&ClientMsg::Play(path))?)?;
        Ok(())
    }

    fn write_all(&mut self, bytes: &[u8]) -> eyre::Result<()> {
        let stream = self.get_stream()?;
        stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
        stream.write_all(&bytes)?;
        Ok(())
    }
}

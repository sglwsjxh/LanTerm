use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use bytes::Bytes;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct Session {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Session {
    pub fn new(shell: &str) -> anyhow::Result<(Self, mpsc::Receiver<Bytes>)> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(std::env::current_dir()?);
        let child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave); // spawn 后释放 slave

        let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
        let reader = pair.master.try_clone_reader()?;
        let (tx, rx) = mpsc::channel::<Bytes>(64);

        // 读线程：spawn_blocking 因为 read 阻塞
        tokio::task::spawn_blocking(move || {
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.blocking_send(Bytes::copy_from_slice(&buf[..n])).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok((
            Self {
                master: Some(pair.master),
                child: Some(child),
                writer,
            },
            rx,
        ))
    }

    pub fn write(&self, data: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(data);
            let _ = w.flush();
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        if let Some(master) = &self.master {
            let _ = master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }

    pub fn kill(&mut self) {
        // 不碰 writer lock（kill 只动 child + master）
        if let Some(mut child) = self.child.take() {
            let _ = child.kill(); // 先 kill
            let _ = child.wait(); // 再 wait（Windows 必须这个顺序）
        }
        // master 显式 drop（让 ConPTY/Unix 管道断开，唤醒阻塞的 read 线程）
        if let Some(master) = self.master.take() {
            drop(master);
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.kill();
    }
}

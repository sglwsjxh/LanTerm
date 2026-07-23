/*
 * LanTerm - Lightweight LAN web terminal sharing
 *
 * Copyright (C) 2026 清木殇
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use bytes::Bytes;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

fn home_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("failed to get cwd"))
}

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
        cmd.cwd(home_dir());
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

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use crate::pty::Session;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum CtrlMsg {
    #[serde(rename = "resize")]
    Resize { cols: u16, rows: u16 },
}

#[derive(Serialize)]
struct ErrorMsg<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    msg: &'a str,
}

pub async fn handler(ws: WebSocketUpgrade, shell: String) -> impl IntoResponse {
    ws.on_upgrade(move |socket| run_session(socket, shell))
}

async fn run_session(socket: WebSocket, shell: String) {
    let (mut sender, mut receiver) = socket.split();

    // 创建 session
    let (session, mut pty_rx) = match Session::new(&shell) {
        Ok(s) => {
            let (sess, rx) = s;
            (Arc::new(Mutex::new(sess)), rx)
        }
        Err(e) => {
            let msg = ErrorMsg {
                kind: "error",
                msg: &format!("{e}"),
            };
            let _ = sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap_or_default()))
                .await;
            let _ = sender.send(Message::Close(None)).await;
            return;
        }
    };

    // PTY 输出 → WS 发送（send_task 不持有 session，只持 pty_rx）
    let mut send_task = tokio::spawn(async move {
        while let Some(bytes) = pty_rx.recv().await {
            if sender.send(Message::Binary(bytes.to_vec())).await.is_err() {
                break;
            }
        }
    });

    // WS 接收 → PTY 写 / resize（write_task clone Arc<Mutex<Session>>）
    let session_for_write = session.clone();
    let mut write_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Binary(d) => {
                    let s = session_for_write.lock().unwrap();
                    s.write(&d);
                }
                Message::Text(t) => {
                    if let Ok(ctrl) = serde_json::from_str::<CtrlMsg>(&t) {
                        match ctrl {
                            CtrlMsg::Resize { cols, rows } => {
                                let s = session_for_write.lock().unwrap();
                                s.resize(cols, rows);
                            }
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // 竞争退出：任一结束 → 另一个 abort → kill session
    tokio::select! {
        _ = &mut send_task => {
            write_task.abort();
        }
        _ = &mut write_task => {
            send_task.abort();
        }
    }

    // kill 不碰 writer lock（pty.rs kill 只 take child + take master）
    session.lock().unwrap().kill();
}

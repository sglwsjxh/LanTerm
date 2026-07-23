# LanTerm

轻量级局域网 Web 终端共享工具

<p align="center">
  <a href="README-zh.md">中文</a> | <a href="README.md">English</a>
</p>

## 构建

### 使用 Makefile

```bash
make build        # 构建前端 + Rust release 二进制文件
make dev          # 构建前端 + Rust debug 版本
make frontend     # 仅构建前端资源
make clean        # 清理构建产物
```

### 手动构建

```bash
cd frontend && npm install && npm run build
cd .. && cargo build --release
# → target/release/lanterm
```

## 运行

```bash
lanterm              # 默认端口 8999
lanterm --port 8080  # 自定义端口
lanterm --shell zsh  # 自定义 shell
```

## 工作原理

LanTerm 在 0.0.0.0:8999 上启动 HTTP 服务器，提供 Vue 3 + xterm.js 前端，并将 WebSocket 连接升级为 portable-pty shell 会话。键盘输入通过 WebSocket 二进制帧发送到 PTY writer；PTY 输出以二进制帧形式流式传输回浏览器

## 已知限制（v0.1）

- 无认证 — 局域网内任何人都可以连接
- 每个连接只有一个终端
- 多网卡机器上局域网 IP 检测可能选错网卡
- 移动端触摸选择有限；通过隐藏的 textarea 输入键盘

## 技术栈

- **后端**：Rust + Axum + tokio + portable-pty + rust-embed
- **前端**：Vue 3 + xterm.js + @xterm/addon-fit
- **协议**：WebSocket（PTY I/O 使用二进制帧，resize 控制使用文本 JSON）

# 许可证

[AGPL-3.0](LICENSE)

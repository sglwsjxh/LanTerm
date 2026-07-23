mod assets;
mod pty;
mod ws;

use axum::{routing::get, Router};
use clap::Parser;
use local_ip_address::local_ip;

#[derive(Parser)]
#[command(name = "lanterm", about = "LAN web terminal sharing")]
struct Cli {
    #[arg(long, default_value = "8999")]
    port: u16,
    #[arg(long)]
    shell: Option<String>,
}

fn default_shell() -> String {
    if cfg!(windows) {
        "powershell.exe".into()
    } else {
        "bash".into()
    }
}

fn detect_lan_ip() -> Option<String> {
    local_ip().ok().map(|ip| ip.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let shell = cli.shell.unwrap_or_else(default_shell);
    let lan_ip = detect_lan_ip().unwrap_or_else(|| "127.0.0.1".into());

    println!(
        "LanTerm running\n\n Local: http://127.0.0.1:{}\n LAN: http://{}:{}\n\n ⚠ no auth — anyone on your LAN can connect\n",
        cli.port, lan_ip, cli.port
    );

    let addr = format!("0.0.0.0:{}", cli.port);
    let shell_clone = shell.clone();

    let app = Router::new()
        .route("/ws", get(move |ws| ws::handler(ws, shell_clone.clone())))
        .fallback(assets::serve_embedded);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

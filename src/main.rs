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

mod assets;
mod pty;
mod ws;

use axum::{routing::get, Router};
use clap::Parser;
use local_ip_address::list_afinet_netifas;

#[derive(Parser)]
#[command(name = "lanterm", version = env!("CARGO_PKG_VERSION"), about = "LAN web terminal sharing")]
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

fn detect_lan_ips() -> Vec<String> {
    list_afinet_netifas()
        .ok()
        .map(|ifas| {
            ifas.into_iter()
                .filter(|(_, ip)| ip.is_ipv4() && !ip.is_loopback())
                .map(|(_, ip)| ip.to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let shell = cli.shell.unwrap_or_else(default_shell);
    let lan_ips = detect_lan_ips();

    println!("LanTerm running\n\n Local: http://127.0.0.1:{}", cli.port);
    for ip in &lan_ips {
        println!("  LAN: http://{}:{}", ip, cli.port);
    }
    println!("\n ⚠ no auth — anyone on your LAN can connect\n");

    let addr = format!("0.0.0.0:{}", cli.port);
    let shell_clone = shell.clone();

    let app = Router::new()
        .route("/ws", get(move |ws| ws::handler(ws, shell_clone.clone())))
        .fallback(assets::serve_embedded);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

pub mod utils {
    pub mod packet;
}
pub mod core {
    pub mod config;
    pub mod proxy;
    pub mod servers;
}

use anyhow::Result;
use crate::core::config::Config;
use crate::utils::packet::{HandshakeRequest, NextState};
use crate::core::servers::Servers;
use crate::core::proxy::ProxyProtocol;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use reqwest::Client;
use std::io::Cursor;
use serde_json::json;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use crate::utils::packet;

#[tokio::main]
async fn main() {
    if let Err(e) = launch_sequence() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let config = load_conf();
    let servers = Servers::load().expect("Failed to load servers.json");
    debug!("Configuration: {:?}", config);

    start(config, servers).await;
}

fn launch_sequence() -> Result<(), Box<dyn std::error::Error>> {
    const LAUNCH_ASCII: &str = r#"
 __  __           _ ____                      
|  \/  | __ _  __| |  _ \ _ __ _____  ___   _ 
| |\/| |/ _` |/ _` | |_) | '__/ _ \ \/ / | | |
| |  | | (_| | (_| |  __/| | | (_) >  <| |_| |
|_|  |_|\__,_|\__,_|_|   |_|  \___/_/\_\\__, |
                                        |___/ "#;

    println!("{}", LAUNCH_ASCII);

    Ok(())
}


async fn start(config: Config, servers: Servers) {
    let listen_addr = config.get_listen_addr();
    info!("Listening on {}", listen_addr);
    let mut listener = TcpListener::bind(listen_addr).await.unwrap();
    
    loop {
        let client = accept_client(&mut listener).await;
        if let Err(e) = client {
            error!("Failed to accept a client: {}", e);
            continue;
        }
        let (stream, addr) = client.unwrap();
        debug!("Client connected from {:?}", addr);
        let config = config.clone();
        let servers = servers.clone();
        tokio::spawn(async move {
            let result = handle_client(&config, &servers, stream, addr).await;
            if let Err(e) = result {
                error!("{}: An error occurred: {}", addr, e);
            }
        });
    }
}

async fn accept_client(listener: &mut TcpListener) -> Result<(TcpStream, SocketAddr)> {
    let client = listener.accept().await?;
    client.0.set_nodelay(true)?;
    Ok(client)
}

async fn handle_client(
    config: &Config, 
    servers: &Servers, 
    mut stream: TcpStream, 
    addr: SocketAddr
) -> Result<()> {
    let handshake = HandshakeRequest::read(&mut stream).await?;
    let host: &str = &handle_hostname(handshake.get_host()).await;
    let server_entry = servers.get_by_hostname(host);

    info!(
        "{}: {}: {}:{} -> {}",
        addr,
        handshake.get_next_state(),
        host,
        handshake.get_port(),
        server_entry
            .as_ref()
            .map(|s| s.backend_server.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );

    if server_entry.is_none() {
        if *handshake.get_next_state() == NextState::Login {
            let mut kick_msg = config.get_unknown_host_kick_msg();
            write_string(&mut stream, &mut &**&mut kick_msg).await?;
        } else if *handshake.get_next_state() == NextState::Status {
            let mut motd = config.get_unknown_host_motd();
            write_string(&mut stream, &mut &**&mut motd).await?;
        }
        return Ok(());
    }

    let server_entry = server_entry.unwrap();
    let server_id = &server_entry.id;

    // Try to connect to the target server
    let server_addr = server_entry.backend_server.to_socket_addrs()?.next().unwrap();
    let server_result = TcpStream::connect(&server_addr).await;

    if let Err(e) = server_result {
        warn!("Failed to connect to backend server: {}", e);
        if *handshake.get_next_state() == NextState::Login {
            if config.auto_start {
                // Try to start the server
                if let Err(e) = start_server(config, server_id).await {
                    error!("Failed to start server: {}", e);
                } else {
                    info!("Server start signal sent for {}", server_id);
                }
                let mut kick_msg = config.get_offline_server_starting_msg();
                write_string(&mut stream, &mut &**&mut kick_msg).await?;
            } else {
                let mut kick_msg = config.get_offline_server_kick_msg();
                write_string(&mut stream, &mut &**&mut kick_msg).await?;
            }
        } else if *handshake.get_next_state() == NextState::Status {
            let mut motd = config.get_offline_server_motd_not_starting(server_id).await;
            write_string(&mut stream, &mut &**&mut motd).await?;
        }
        return Ok(());
    }

    let mut server = server_result.unwrap();
    server.set_nodelay(true)?;

    // Send PROXY protocol header
    let proxy = ProxyProtocol::new(addr, server_addr);
    let header = proxy.generate_header();
    server.write_all(&header).await?;

    // Send Minecraft handshake
    packet::write_var_int(&mut server, handshake.get_size()).await?;
    server.write_all(handshake.get_raw_body()).await?;

    let (mut client_reader, mut client_writer) = tokio::io::split(stream);
    let (mut server_reader, mut server_writer) = tokio::io::split(server);
    
    tokio::spawn(async move {
        let result = tokio::io::copy(&mut client_reader, &mut server_writer).await;
        if let Some(err) = result.err() {
            debug!(
                "{}: An error occurred in client-to-server bridge. Maybe disconnected: {}",
                addr, err
            );
        }
    });
    
    let result = tokio::io::copy(&mut server_reader, &mut client_writer).await;
    if let Some(err) = result.err() {
        debug!(
            "{}: An error occurred in server-to-client bridge. Maybe disconnected: {}",
            addr, err
        );
    }
    Ok(())
}

async fn handle_hostname(hostname: &str) -> String {
    let mut host: String = hostname.to_owned();

    // TCPShield Support (UNTESTED!)
    if host.contains("///") {
        let parts = host.split("///");
        for part in parts {
            host = part.to_owned();
            break;
        }
    }

    // Forge Support
    if host.contains("FML2") {
        host = host.replace("FML2", "");
    } else if host.contains("FML") {
        host = host.replace("FML", "");
    }
    host
}

async fn write_string(stream: &mut TcpStream, string: &mut &str) -> Result<()> {
    let mut temp: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    crate::utils::packet::write_var_int(&mut temp, 0).await?;
    crate::utils::packet::write_var_int(&mut temp, string.len() as i32).await?;
    temp.write_all(&string.as_bytes()).await?;
    let temp = temp.into_inner();
    crate::utils::packet::write_var_int(stream, temp.len() as i32).await?;
    stream.write_all(&temp).await?;
    Ok(())
}

fn load_conf() -> Config {
    let config_path = Path::new("./config.yml");
    info!("Configuration file: {:?}", config_path);
    Config::load_or_init(config_path)
}

async fn start_server(config: &Config, server_id: &str) -> Result<()> {
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", config.api_key()))?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let url = format!("{}/api/client/servers/{}/power", config.panel_link(), server_id);
    
    client.post(url)
        .headers(headers)
        .json(&json!({
            "signal": "start"
        }))
        .send()
        .await?;

    Ok(())
}

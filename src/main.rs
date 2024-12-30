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
}

use anyhow::Result;
use crate::core::config::Config;
use crate::utils::packet::{HandshakeRequest, NextState};
use crate::core::proxy::ProxyProtocol;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use std::io::Cursor;
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
    debug!("Configuration: {:?}", config);

    start(config).await;
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

async fn start(config: Config) {
    let port_range = config.port_range.clone();

    for port in port_range.start..=port_range.end {
        let listen_addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&listen_addr).await.unwrap();
        info!("Listening on {}", listen_addr);

        let config = config.clone();
        tokio::spawn(async move {
            loop {
                match accept_client(&listener).await {
                    Ok((stream, addr)) => {
                        debug!("Client connected from {:?}", addr);
                        let config = config.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(&config, stream, addr, port).await {
                                error!("{}: An error occurred: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept a client: {}", e);
                    }
                }
            }
        });
    }

    // Prevent the main function from exiting immediately
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn accept_client(listener: &TcpListener) -> Result<(TcpStream, SocketAddr)> {
    let client = listener.accept().await?;
    client.0.set_nodelay(true)?;
    Ok(client)
}

async fn handle_client(
    config: &Config, 
    mut stream: TcpStream, 
    addr: SocketAddr,
    port: u16
) -> Result<()> {
    let handshake = HandshakeRequest::read(&mut stream).await?;
    let server_addr = format!("{}:{}", config.backend_server, port);
    let server_result = TcpStream::connect(&server_addr).await;

    if let Err(e) = server_result {
        warn!("Failed to connect to backend server: {}", e);
        if *handshake.get_next_state() == NextState::Login {
            let mut kick_msg = config.get_offline_server_kick_msg();
            write_string(&mut stream, &mut &**&mut kick_msg).await?;
        } else if *handshake.get_next_state() == NextState::Status {
            let mut motd = config.get_offline_server_motd_not_starting("").await;
            write_string(&mut stream, &mut &**&mut motd).await?;
        }
        return Ok(());
    }

    let mut server = server_result.unwrap();
    server.set_nodelay(true)?;

    // Send PROXY protocol header
    let proxy = ProxyProtocol::new(addr, server.peer_addr().unwrap());
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

fn load_conf() -> Config {
    let config_path = Path::new("./config.yml");
    info!("Configuration file: {:?}", config_path);
    Config::load_or_init(config_path)
}

async fn write_string(stream: &mut TcpStream, string: &str) -> Result<()> {
    let mut temp: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    packet::write_var_int(&mut temp, 0).await?;
    packet::write_var_int(&mut temp, string.len() as i32).await?;
    temp.write_all(string.as_bytes()).await?;
    let temp = temp.into_inner();
    packet::write_var_int(stream, temp.len() as i32).await?;
    stream.write_all(&temp).await?;
    Ok(())
}

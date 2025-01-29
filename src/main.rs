use std::future::IntoFuture;

use axum::{routing::get, Router};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    signal,
    sync::broadcast,
    task,
};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");
    println!("Server listening on {}", addr);
    let listener_http = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // Broadcast channel for shutdown signal
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get("Hello"));
    // `POST /users` goes to `create_user`

    // run our app with hyper, listening globally on port 3000

    let http_server = axum::serve(listener_http, app);
    tokio::spawn(http_server.into_future());

    // Accept incoming connections
    loop {
        tokio::select! {
            Ok((socket, addr)) = listener.accept() => {
                println!("New connection from {}", addr);
                let shutdown_rx_conn = shutdown_tx.subscribe();
                task::spawn(handle_connection(socket, shutdown_rx_conn));
            },
            _ = shutdown_rx.recv() => {
                println!("Shutting down server...");
                break;
            }
            _ = signal::ctrl_c() => {
                shutdown_tx
                .send(())
                .expect("Sending shutdown signal failed");
            },
        }
    }
}

// Handles incoming TCP connections
async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let (reader, mut writer) = socket.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = buf_reader.read_line(&mut line) => {
                if let Ok(n) = result {
                    if n == 0 {
                        break; // Client disconnected
                    }
                    println!("Received: {}", line.trim());
                    writer.write_all(b"Command received\n").await.unwrap();
                    line.clear();
                }
            },
            _ = shutdown_rx.recv() => {
                println!("Closing connection...");
                writer.write_all(b"Shutdown started. Closing connection...\n").await.unwrap();
                break;
            }
        }
    }
}

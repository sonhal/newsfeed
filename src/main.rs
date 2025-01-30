use serde::Serialize;
use std::{env, future::IntoFuture, sync::Arc};

use axum::{extract::State, routing::get, Json, Router};
use axum_macros::debug_handler;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    signal,
    sync::broadcast,
    task,
};
use tokio_postgres::{Client, NoTls};

struct Config {
    http_server_addr: String,
    database_url: String,
}

impl Config {
    fn from_env() -> Config {
        // Read a required variable (panics if missing)
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment");
        let http_server_addr = env::var("HTTP_SERVER_ADDR")
            .expect("HTTP_SERVER_ADDRASE_URL must be set in the environment");
        Config {
            http_server_addr,
            database_url,
        }
    }
}

struct AppState {
    config: Config,
    db_client: Client,
}

#[derive(Serialize)]
struct Subscribed {
    id: i32,
    name: String,
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind TCP listener");
    println!("Server listening on {}", config.http_server_addr);
    let listener_http = tokio::net::TcpListener::bind(&config.http_server_addr)
        .await
        .unwrap();

    // Broadcast channel for shutdown signal
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);

    // setup database connection
    let (client, connection) = tokio_postgres::connect(config.database_url.as_str(), NoTls)
        .await
        .expect("failed to connect to db");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // set up application state
    let shared_state = Arc::new(AppState {
        config: config,
        db_client: client,
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get("Hello"))
        .route("/subscribed", get(get_items))
        .with_state(shared_state);
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

#[debug_handler]
async fn get_items(State(state): State<Arc<AppState>>) -> Json<Vec<Subscribed>> {
    let rows = state
        .db_client
        .query("SELECT id, name FROM subscribed", &[])
        .await
        .unwrap();

    let items: Vec<Subscribed> = rows
        .iter()
        .map(|row| Subscribed {
            id: row.get(0),
            name: row.get(1),
        })
        .collect();

    Json(items)
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

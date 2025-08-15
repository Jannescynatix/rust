use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::{mpsc::{self, Sender}, broadcast};
use std::sync::Arc;
use std::time::Instant;
use serde::Deserialize;
use serde_json::json;
use tokio_util::task::CancellationToken;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server läuft auf {}", addr);

    let (cancel_tx, _) = broadcast::channel(1);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/ws", get(websocket_handler))
        .with_state(Arc::new(AppState { cancel_tx: cancel_tx.clone() }));

    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct AppState {
    cancel_tx: broadcast::Sender<()>,
}

#[derive(Deserialize)]
struct WebSocketMessage {
    r#type: String,
    number: Option<u64>,
}

async fn serve_index() -> impl IntoResponse {
    let html = std::fs::read_to_string("public/index.html")
        .expect("Konnte index.html nicht laden");
    axum::response::Html(html)
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Erstelle einen Channel für Fortschritts-Updates
    let (progress_tx, mut progress_rx) = mpsc::channel::<String>(100);

    // Klonen des Cancel-Senders, um den Abbruch in der Berechnung auszulösen
    let cancel_tx = state.cancel_tx.clone();
    let mut cancel_rx = cancel_tx.subscribe();

    // Empfange Nachrichten vom Client
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg { msg } else { return; };

        if let Message::Text(text) = msg {
            if let Ok(data) = serde_json::from_str::<WebSocketMessage>(&text) {
                match data.r#type.as_str() {
                    "startCalculation" => {
                        if let Some(number) = data.number {
                            let worker_progress_tx = progress_tx.clone();
                            let worker_cancel_rx = cancel_tx.subscribe();

                            tokio::spawn(async move {
                                let start_time = Instant::now();
                                let (factors, _last_divisor) = find_prime_factors(number, worker_progress_tx, worker_cancel_rx).await;
                                let duration = start_time.elapsed();
                                let duration_ms = duration.as_millis();
                                let duration_sec = duration.as_secs_f64();

                                let response = json!({
                                    "type": "done",
                                    "number": number,
                                    "factors": factors,
                                    "durationMs": format!("{:.2}", duration_ms),
                                    "durationSec": format!("{:.2}", duration_sec),
                                });
                                let _ = progress_tx.send(response.to_string()).await;
                            });

                            while let Some(message) = progress_rx.recv().await {
                                let _ = socket.send(Message::Text(message)).await;
                            }
                        }
                    },
                    "cancelCalculation" => {
                        let _ = cancel_tx.send(());
                        println!("Calculation cancelled by client.");
                    },
                    _ => {},
                }
            }
        }
    }
}

async fn find_prime_factors(mut n: u64, sender: mpsc::Sender<String>, mut cancellation: broadcast::Receiver<()>) -> (Vec<u64>, u64) {
    let mut factors = vec![];
    let mut d = 2;
    let limit = (n as f64).sqrt() as u64;

    while n >= 2 && d <= limit {
        // Prüfe, ob eine Abbruch-Nachricht empfangen wurde
        if cancellation.try_recv().is_ok() {
            let _ = sender.send(json!({"type": "cancelled", "message": "Berechnung wurde abgebrochen."}).to_string()).await;
            return (factors, d);
        }

        if n % d == 0 {
            factors.push(d);
            n /= d;
        } else {
            d += 1;
        }

        // Sende Fortschritts-Updates nur alle 1000 Iterationen
        if d % 1000 == 0 {
            let progress = (d as f64 / limit as f64) * 100.0;
            let _ = sender.send(json!({"type": "progress", "progress": progress.round()}).to_string()).await;
        }
    }

    if n > 1 {
        factors.push(n);
    }

    (factors, d)
}
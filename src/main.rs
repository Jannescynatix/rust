// Diese Funktion ersetzt die alte handle_socket-Funktion
async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Channel für Fortschritts-Updates einmalig außerhalb der Schleife erstellen
    let (progress_tx, mut progress_rx) = mpsc::channel::<String>(100);
    let cancel_tx = state.cancel_tx.clone();

    // Empfange Nachrichten vom Client
    loop {
        // Empfange Nachrichten vom Socket oder von der Fortschritts-Schleife
        tokio::select! {
            // Neuer Message-Sender für jeden Task
            Some(msg) = socket.recv() => {
                let msg = if let Ok(msg) = msg { msg } else { return; };

                if let Message::Text(text) = msg {
                    if let Ok(data) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match data.r#type.as_str() {
                            "startCalculation" => {
                                if let Some(number) = data.number {
                                    // Klonen der Sender für den neuen Task
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
            // Weiterleiten von Fortschrittsnachrichten an den Client
            Some(message) = progress_rx.recv() => {
                let _ = socket.send(Message::Text(message)).await;
            }
            else => {
                // Die Schleife beenden, wenn der Socket geschlossen wird
                return;
            }
        }
    }
}
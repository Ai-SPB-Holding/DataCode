use crate::interpreter::Interpreter;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};

pub mod output_capture;

use output_capture::OutputCapture;

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteRequest {
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExecuteResponse {
    success: bool,
    output: String,
    error: Option<String>,
}

/// Запустить WebSocket сервер на указанном адресе
pub async fn start_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address).await?;
    println!("🚀 DataCode WebSocket Server запущен на {}", address);
    println!("📡 Ожидание подключений...");
    println!("💡 Отправьте JSON запрос: {{\"code\": \"ваш код\"}}");
    println!("💡 Ответ будет в формате: {{\"success\": true/false, \"output\": \"...\", \"error\": null/\"...\"}}");
    println!();

    // Используем LocalSet для локальных задач, так как Interpreter не является Send
    let local_set = tokio::task::LocalSet::new();
    
    // Создаем listener внутри LocalSet и обрабатываем подключения
    local_set.run_until(async {
        loop {
            let (stream, addr) = match listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(e) => {
                    eprintln!("❌ Ошибка принятия подключения: {}", e);
                    continue;
                }
            };
            
            println!("✅ Новое подключение от {}", addr);
            local_set.spawn_local(handle_client(stream));
        }
    }).await;

    Ok(())
}

/// Обработать клиентское подключение
async fn handle_client(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("❌ Ошибка при принятии WebSocket соединения: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    // Создаем отдельный интерпретатор для каждого клиента
    let mut interpreter = Interpreter::new();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Парсим запрос
                let request: ExecuteRequest = match serde_json::from_str(&text) {
                    Ok(req) => req,
                    Err(e) => {
                        let error_response = ExecuteResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Ошибка парсинга запроса: {}", e)),
                        };
                        if let Ok(json) = serde_json::to_string(&error_response) {
                            let _ = write.send(Message::Text(json)).await;
                        }
                        continue;
                    }
                };

                // Выполняем код (синхронно, так как Interpreter не является Send)
                let response = execute_code(&mut interpreter, &request.code);
                
                // Отправляем ответ
                if let Ok(json) = serde_json::to_string(&response) {
                    if let Err(e) = write.send(Message::Text(json)).await {
                        eprintln!("❌ Ошибка отправки ответа: {}", e);
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("🔌 Клиент отключился");
                break;
            }
            Ok(Message::Ping(data)) => {
                if let Err(e) = write.send(Message::Pong(data)).await {
                    eprintln!("❌ Ошибка отправки Pong: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ Ошибка чтения сообщения: {}", e);
                break;
            }
            _ => {}
        }
    }
}

/// Выполнить код и вернуть результат
fn execute_code(
    interpreter: &mut Interpreter,
    code: &str,
) -> ExecuteResponse {
    // Создаем буфер для перехвата вывода
    let output_capture = OutputCapture::new();
    
    // Устанавливаем буфер для текущего потока
    output_capture.set_capture(true);

    // Выполняем код
    let result = interpreter.exec(code);

    // Получаем вывод
    let output = output_capture.get_output();
    output_capture.set_capture(false);

    // Формируем ответ
    match result {
        Ok(()) => ExecuteResponse {
            success: true,
            output,
            error: None,
        },
        Err(e) => ExecuteResponse {
            success: false,
            output,
            error: Some(e.to_string()),
        },
    }
}


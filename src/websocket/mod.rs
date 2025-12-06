use crate::interpreter::Interpreter;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

pub mod output_capture;
pub mod smb;

use output_capture::OutputCapture;
use smb::{SmbManager, SmbConnection};
use crate::builtins::file::{set_smb_manager, clear_smb_manager};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WebSocketRequest {
    #[serde(rename = "execute")]
    Execute { code: String },
    #[serde(rename = "smb_connect")]
    SmbConnect {
        ip: String,
        login: String,
        password: String,
        domain: String,
        share_name: String,
    },
    #[serde(rename = "smb_list_files")]
    SmbListFiles {
        share_name: String,
        path: String,
    },
    #[serde(rename = "smb_read_file")]
    SmbReadFile {
        share_name: String,
        file_path: String,
    },
}

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

#[derive(Debug, Serialize, Deserialize)]
struct SmbConnectResponse {
    success: bool,
    message: String,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SmbListFilesResponse {
    success: bool,
    files: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SmbReadFileResponse {
    success: bool,
    content: Option<String>,
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
    // Создаем отдельный SmbManager для каждого клиента
    let smb_manager = Arc::new(Mutex::new(SmbManager::new()));
    
    // Устанавливаем SmbManager в thread-local storage для доступа из функций файловых операций
    set_smb_manager(smb_manager.clone());

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Пытаемся распарсить как новый формат с типом команды
                if let Ok(request) = serde_json::from_str::<WebSocketRequest>(&text) {
                    match request {
                        WebSocketRequest::Execute { code } => {
                            // Выполняем код (синхронно, так как Interpreter не является Send)
                            let response = execute_code(&mut interpreter, &code, &smb_manager);
                            
                            // Отправляем ответ
                            if let Ok(json) = serde_json::to_string(&response) {
                                if let Err(e) = write.send(Message::Text(json)).await {
                                    eprintln!("❌ Ошибка отправки ответа: {}", e);
                                    break;
                                }
                            }
                        }
                        WebSocketRequest::SmbConnect { ip, login, password, domain, share_name } => {
                            let connection = SmbConnection::new(ip, login, password, domain, share_name);
                            let result = smb_manager.lock().unwrap().connect(connection);
                            
                            let response = match result {
                                Ok(msg) => SmbConnectResponse {
                                    success: true,
                                    message: msg,
                                    error: None,
                                },
                                Err(e) => SmbConnectResponse {
                                    success: false,
                                    message: String::new(),
                                    error: Some(e),
                                },
                            };
                            
                            if let Ok(json) = serde_json::to_string(&response) {
                                if let Err(e) = write.send(Message::Text(json)).await {
                                    eprintln!("❌ Ошибка отправки ответа: {}", e);
                                    break;
                                }
                            }
                        }
                        WebSocketRequest::SmbListFiles { share_name, path } => {
                            let result = smb_manager.lock().unwrap().list_files(&share_name, &path);
                            
                            let response = match result {
                                Ok(files) => SmbListFilesResponse {
                                    success: true,
                                    files,
                                    error: None,
                                },
                                Err(e) => SmbListFilesResponse {
                                    success: false,
                                    files: Vec::new(),
                                    error: Some(e),
                                },
                            };
                            
                            if let Ok(json) = serde_json::to_string(&response) {
                                if let Err(e) = write.send(Message::Text(json)).await {
                                    eprintln!("❌ Ошибка отправки ответа: {}", e);
                                    break;
                                }
                            }
                        }
                        WebSocketRequest::SmbReadFile { share_name, file_path } => {
                            let result = smb_manager.lock().unwrap().read_file(&share_name, &file_path);
                            
                            let response = match result {
                                Ok(content) => {
                                    // Пытаемся декодировать как UTF-8, если не получается - возвращаем base64
                                    match String::from_utf8(content.clone()) {
                                        Ok(text) => SmbReadFileResponse {
                                            success: true,
                                            content: Some(text),
                                            error: None,
                                        },
                                        Err(_) => {
                                            // Если не UTF-8, возвращаем base64
                                            use base64::Engine;
                                            let base64_content = base64::engine::general_purpose::STANDARD.encode(&content);
                                            SmbReadFileResponse {
                                                success: true,
                                                content: Some(format!("base64:{}", base64_content)),
                                                error: None,
                                            }
                                        }
                                    }
                                }
                                Err(e) => SmbReadFileResponse {
                                    success: false,
                                    content: None,
                                    error: Some(e),
                                },
                            };
                            
                            if let Ok(json) = serde_json::to_string(&response) {
                                if let Err(e) = write.send(Message::Text(json)).await {
                                    eprintln!("❌ Ошибка отправки ответа: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    // Пытаемся распарсить как старый формат для обратной совместимости
                    if let Ok(request) = serde_json::from_str::<ExecuteRequest>(&text) {
                        let response = execute_code(&mut interpreter, &request.code, &smb_manager);
                        
                        if let Ok(json) = serde_json::to_string(&response) {
                            if let Err(e) = write.send(Message::Text(json)).await {
                                eprintln!("❌ Ошибка отправки ответа: {}", e);
                                break;
                            }
                        }
                    } else {
                        let error_response = ExecuteResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Ошибка парсинга запроса. Ожидается JSON с полями: type, code (или smb_connect, smb_list_files, smb_read_file)")),
                        };
                        if let Ok(json) = serde_json::to_string(&error_response) {
                            let _ = write.send(Message::Text(json)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("🔌 Клиент отключился");
                // Отключаем все SMB подключения при отключении клиента
                let mut manager = smb_manager.lock().unwrap();
                let shares: Vec<String> = manager.list_connections();
                for share in shares {
                    let _ = manager.disconnect(&share);
                }
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
    
    // Очищаем thread-local storage
    clear_smb_manager();
}

/// Выполнить код и вернуть результат
fn execute_code(
    interpreter: &mut Interpreter,
    code: &str,
    smb_manager: &Arc<Mutex<SmbManager>>,
) -> ExecuteResponse {
    // Устанавливаем SmbManager в thread-local storage для доступа из функций файловых операций
    set_smb_manager(smb_manager.clone());
    
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


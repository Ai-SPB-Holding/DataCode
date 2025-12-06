import asyncio
import websockets
import json
import sys
import os 

username = "username"
password = "password"

smb_server = "smb_server"
smb_share = "smb_share"
domain = ""  # Обычно WORKGROUP для Windows или имя домена

ws_url = "ws://0.0.0.0:8899"

async def connect_and_execute(datacode):
    """Подключиться к WebSocket и выполнить SMB подключение и DataCode скрипт"""
    try:
        print(f"🔌 Подключение к WebSocket серверу: {ws_url}")
        async with websockets.connect(ws_url) as websocket:
            print("✅ Подключено к WebSocket серверу")
            
            # 1. Подключение к SMB шаре
            print(f"\n📡 Отправка запроса на подключение к SMB шаре '{smb_share}'...")
            smb_connect_request = {
                "type": "smb_connect",
                "ip": smb_server,
                "login": username,
                "password": password,
                "domain": domain,
                "share_name": smb_share
            }
            
            await websocket.send(json.dumps(smb_connect_request))
            print(f"📤 Отправлен запрос: {json.dumps(smb_connect_request, indent=2)}")
            
            # Получаем ответ о подключении
            response = await websocket.recv()
            smb_response = json.loads(response)
            print(f"\n📥 Ответ сервера:")
            print(json.dumps(smb_response, indent=2, ensure_ascii=False))
            
            if smb_response.get("success"):
                print(f"✅ Успешно подключено к SMB шаре '{smb_share}'")
            else:
                error = smb_response.get("error", "Неизвестная ошибка")
                print(f"❌ Ошибка подключения: {error}")
                return
            
            # 2. Выполнение DataCode скрипта
            print(f"\n📡 Выполнение DataCode скрипта...")
            print(f"📝 Код:\n{datacode}\n")
            
            execute_request = {
                "type": "execute",
                "code": datacode
            }
            
            await websocket.send(json.dumps(execute_request))
            print("📤 Отправлен запрос на выполнение кода")
            
            # Получаем ответ с результатами выполнения
            response = await websocket.recv()
            execute_response = json.loads(response)
            print(f"\n📥 Результат выполнения:")
            print(json.dumps(execute_response, indent=2, ensure_ascii=False))
            
            if execute_response.get("success"):
                print(f"\n✅ Код выполнен успешно")
                if execute_response.get("output"):
                    print(f"\n📋 Вывод:\n{execute_response['output']}")
            else:
                error = execute_response.get("error", "Неизвестная ошибка")
                print(f"\n❌ Ошибка выполнения: {error}")
                if execute_response.get("output"):
                    print(f"📋 Вывод:\n{execute_response['output']}")
                    
    except ConnectionRefusedError:
        print(f"❌ Не удалось подключиться к {ws_url}")
        print("💡 Убедитесь, что WebSocket сервер запущен")
        print("💡 Запустите сервер командой: datacode --websocket --host 0.0.0.0 --port 8899")
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    # Проверка аргументов командной строки
    if len(sys.argv) < 2:
        print("❌ Ошибка: не указан файл .dc")
        print("💡 Использование: python3 test_smb_connection.py <файл.dc>")
        sys.exit(1)
    
    dc_file = sys.argv[1]
    
    # Проверка расширения файла
    if not dc_file.endswith('.dc'):
        print(f"❌ Ошибка: файл должен иметь расширение .dc")
        print(f"💡 Получен файл: {dc_file}")
        sys.exit(1)
    
    # Проверка существования файла
    if not os.path.exists(dc_file):
        print(f"❌ Ошибка: файл не найден: {dc_file}")
        sys.exit(1)
    
    # Чтение содержимого файла
    try:
        with open(dc_file, 'r', encoding='utf-8') as f:
            datacode = f.read()
        print(f"📄 Загружен файл: {dc_file}")
        print(f"📏 Размер кода: {len(datacode)} символов\n")
    except Exception as e:
        print(f"❌ Ошибка при чтении файла {dc_file}: {e}")
        sys.exit(1)
    
    print("🚀 Запуск теста SMB подключения через WebSocket\n")
    asyncio.run(connect_and_execute(datacode))
#!/usr/bin/env python3
"""
Тестовый скрипт для загрузки файлов через WebSocket сервер DataCode
Требуется: pip install websockets

Важно: Сервер должен быть запущен с флагом --use-ve:
    datacode --websocket --host 0.0.0.0 --port 8899 --use-ve
"""

import asyncio
import websockets
import json
import base64
import os
from pathlib import Path

async def test_file_upload():
    uri = "ws://127.0.0.1:8899"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("✅ Подключено к серверу")
            print("💡 Убедитесь, что сервер запущен с флагом --use-ve")
            print()
            
            # Тест 1: Проверка getcwd() - должен вернуть пустую строку для безопасности
            print("📋 Тест 1: Проверка getcwd() (должен вернуть пустую строку для безопасности)")
            test1 = {
                "type": "execute",
                "code": "global cwd = getcwd()\nprint('Current directory:', cwd)\nprint('Type of cwd:', typeof(cwd))"
            }
            print(f"📤 Отправка: {json.dumps(test1, ensure_ascii=False)}")
            await websocket.send(json.dumps(test1))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Output: {result['output']}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 2: Загрузка текстового файла
            print("📋 Тест 2: Загрузка текстового файла")
            text_content = """Hello, DataCode!
This is a test file uploaded via WebSocket.
Line 3 of the file.
"""
            upload_text = {
                "type": "upload_file",
                "filename": "test.txt",
                "content": text_content
            }
            print(f"📤 Отправка файла: test.txt ({len(text_content)} байт)")
            await websocket.send(json.dumps(upload_text))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Message: {result.get('message', '')}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 3: Загрузка CSV файла
            print("📋 Тест 3: Загрузка CSV файла")
            csv_content = """name,age,city
Alice,30,New York
Bob,25,London
Charlie,35,Paris
"""
            upload_csv = {
                "type": "upload_file",
                "filename": "data.csv",
                "content": csv_content
            }
            print(f"📤 Отправка файла: data.csv ({len(csv_content)} байт)")
            await websocket.send(json.dumps(upload_csv))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Message: {result.get('message', '')}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 4: Загрузка файла в поддиректории
            print("📋 Тест 4: Загрузка файла в поддиректории")
            subdir_content = "This file is in a subdirectory\n"
            upload_subdir = {
                "type": "upload_file",
                "filename": "subdir/nested_file.txt",
                "content": subdir_content
            }
            print(f"📤 Отправка файла: subdir/nested_file.txt")
            await websocket.send(json.dumps(upload_subdir))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Message: {result.get('message', '')}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 5: Загрузка бинарного файла (base64)
            print("📋 Тест 5: Загрузка бинарного файла (base64)")
            # Создаем простой PNG файл (1x1 пиксель, прозрачный)
            png_data = base64.b64encode(
                bytes.fromhex('89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000a49444154789c6300010000000500010d0a2db40000000049454e44ae426082')
            ).decode('utf-8')
            
            upload_binary = {
                "type": "upload_file",
                "filename": "image.png",
                "content": f"base64:{png_data}"
            }
            print(f"📤 Отправка файла: image.png (base64, {len(png_data)} символов)")
            await websocket.send(json.dumps(upload_binary))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Message: {result.get('message', '')}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 6: Чтение загруженного CSV файла через DataCode
            print("📋 Тест 6: Чтение загруженного CSV файла через DataCode")
            read_csv_code = """
# Поскольку getcwd() возвращает пустую строку, используем относительные пути
# Файлы загружаются в папку сессии пользователя
global data = read_file(path("data.csv"), 0)
print("Загружено строк:", len(data))
table_info(data)
"""
            read_csv = {
                "type": "execute",
                "code": read_csv_code
            }
            print(f"📤 Выполнение кода для чтения CSV")
            await websocket.send(json.dumps(read_csv))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Output: {result['output']}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 7: Работа с несколькими файлами
            print("📋 Тест 7: Работа с несколькими загруженными файлами")
            multi_file_code = """
# Читаем текстовый файл
global text = read_file(path("test.txt"))
print("Содержимое test.txt:")
print(text)

# Читаем CSV файл
global csv_data = read_file(path("data.csv"))
print("Количество строк в CSV:", len(csv_data))
"""
            multi_file = {
                "type": "execute",
                "code": multi_file_code
            }
            print(f"📤 Выполнение кода для работы с несколькими файлами")
            await websocket.send(json.dumps(multi_file))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Output: {result['output']}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 8: Загрузка папки с данными разных типов и перебор через цикл
            print("📋 Тест 8: Загрузка папки с данными разных типов")
            data_dir = "data_dir"
            
            # Загружаем текстовый файл
            print(f"📤 Загрузка файлов в папку {data_dir}/...")
            files_to_upload = [
                ("data_dir/readme.txt", "This is a text file.\nLine 2 of text file."),
                ("data_dir/data.csv", "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300"),
                ("data_dir/sample.xlsx", None),  # Будет создан как base64
                ("data_dir/image.png", None),  # Будет создан как base64
            ]
            
            # Создаем простой XLSX файл (минимальный валидный XLSX)
            # Это минимальный XLSX файл с одной ячейкой
            xlsx_minimal = base64.b64encode(
                bytes.fromhex(
                    '504b030414000000080000002100000000000000000000000000000000100000'
                    '786c2f776f726b626f6f6b2e786d6c3c3f786d6c2076657273696f6e3d22312e30'
                    '2220656e636f64696e673d225554462d38223f3e3c776f726b626f6f6b20786d6c'
                    '6e733d22687474703a2f2f736368656d61732e6f70656e786d6c666f726d617473'
                    '2e6f72672f73707265616473686565746d6c2f323030362f6d61696e223e3c736865'
                    '6574733e3c7368656574206e616d653d22536865657431222f3e3c2f7368656574'
                    '733e3c2f776f726b626f6f6b3e504b050600000000010001005a0000000000000000'
                    '000000'
                )
            ).decode('utf-8')
            
            # Создаем простой PNG файл (1x1 пиксель)
            png_data = base64.b64encode(
                bytes.fromhex('89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000a49444154789c6300010000000500010d0a2db40000000049454e44ae426082')
            ).decode('utf-8')
            
            for filename, content in files_to_upload:
                if content is None:
                    if "xlsx" in filename:
                        content = f"base64:{xlsx_minimal}"
                    elif "png" in filename:
                        content = f"base64:{png_data}"
                
                upload_req = {
                    "type": "upload_file",
                    "filename": filename,
                    "content": content if not content.startswith("base64:") else content
                }
                await websocket.send(json.dumps(upload_req))
                response = await websocket.recv()
                result = json.loads(response)
                if result.get('success'):
                    print(f"  ✅ {filename}")
                else:
                    print(f"  ❌ {filename}: {result.get('error', 'Unknown error')}")
            
            print()
            
            # Тест 9: Перебор файлов в папке через цикл (с getcwd())
            print("📋 Тест 9: Перебор файлов в папке через цикл list_files (с getcwd())")
            list_files_code = f"""
# В режиме --use-ve getcwd() возвращает пустую строку для безопасности
# Но относительные пути автоматически разрешаются относительно папки сессии
global current_dir = getcwd()
print("Текущая директория (getcwd()): '", current_dir, "'")

# Используем относительный путь - он автоматически разрешится относительно папки сессии
global dir_path = path("{data_dir}")
print("Путь к папке (относительный):", dir_path)

global files = list_files(dir_path)

print("\\nФайлы в папке {data_dir}:")
for file in files do
    print("  -", file)
next file

print("\\nВсего файлов:", len(files))
"""
            list_files_request = {
                "type": "execute",
                "code": list_files_code
            }
            print(f"📤 Выполнение кода для перебора файлов")
            await websocket.send(json.dumps(list_files_request))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Output: {result['output']}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            # Тест 10: Обработка файлов разных типов
            print("📋 Тест 10: Обработка файлов разных типов из папки")
            process_files_code = f"""
# Используем относительный путь - он автоматически разрешится относительно папки сессии
global dir_path = path("{data_dir}")
global files = list_files(dir_path)

print("Обработка файлов:")
for file in files do
    print("Файл:", file)

    if not file.is_file do
        next file
    endif
    
    # Определяем тип файла по расширению
    if file.extension == "txt" do
        global content = read_file(file)
        print("  Тип: Текстовый файл")
        print("  Содержимое:", content)
    endif
    
    if file.extension == "csv" do
        global csv_data = read_file(file, 0)
        print("  Тип: CSV файл")
        print("  Строк:", len(csv_data))
        if len(csv_data) > 0 do
            print("  Первая строка:", csv_data.idx[0])
        endif
    endif
    
    if file.extension == "xlsx" do
        print("  Тип: Excel файл")
        print("  (Excel файлы требуют специальной обработки)")
    endif
    
    if file.extension == "png" do
        print("  Тип: Изображение PNG")
        print("  (Бинарные файлы загружены успешно)")
    endif
next file
"""
            process_files_request = {
                "type": "execute",
                "code": process_files_code
            }
            print(f"📤 Выполнение кода для обработки файлов разных типов")
            await websocket.send(json.dumps(process_files_request))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Output: {result['output']}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()

            print("📋 Тест 11: Проверка списка файлов в папке")
            list_files_code = """
            print("Файлы в папке getcwd():")
            for file in list_files(getcwd()) do
                print("  -", file)
            next file

            print()
            print("Файлы в папке '.':")

            for file in list_files(".") do
                print("  -", file)
            next file

            try
                print("Файлы в папке '..' (должно быть ошибка):")
                for file in list_files("..") do
                    print("  -", file)
                next file
            catch e
                print("Error: ", e)
                print("Должно быть ошибка")
            endtry
            try
                print("Файлы в папке '../' (должно быть ошибка):")
                for file in list_files("../") do
                    print("  -", file)
                next file
            catch e
                print("Error: ", e)
                print("Должно быть ошибка")
            endtry
            try
                print("Файлы в папке '../getcwd()' (должно быть ошибка):")
                for file in list_files(".." / getcwd()) do
                    print("  -", file)
                next file
            catch e
                print("Error: ", e)
                print("Должно быть ошибка")
            endtry
            try
                print("Файлы в папке '../..' (должно быть ошибка):")
                for file in list_files("../..") do
                    print("  -", file)
                next file
            catch e
                print("Error: ", e)
                print("Должно быть ошибка")
            endtry

            """
            list_files_request = {
                "type": "execute",
                "code": list_files_code
            }
            print(f"📤 Выполнение кода для проверки списка файлов")
            await websocket.send(json.dumps(list_files_request))
            
            response = await websocket.recv()
            result = json.loads(response)
            print(f"📥 Получен ответ:")
            print(f"  Success: {result['success']}")
            print(f"  Output: {result['output']}")
            if result.get('error'):
                print(f"  Error: {result['error']}")
            print()
            
            print("✅ Все тесты завершены")
            print("💡 Папка сессии будет автоматически удалена при отключении")
            
    except websockets.exceptions.ConnectionRefused:
        print("❌ Ошибка: Не удалось подключиться к серверу")
        print("💡 Убедитесь, что сервер запущен с флагом --use-ve:")
        print("   datacode --websocket --host 0.0.0.0 --port 8899 --use-ve")
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        import traceback
        traceback.print_exc()

def upload_file_from_disk(websocket, file_path, target_filename=None):
    """
    Вспомогательная функция для загрузки файла с диска
    
    Args:
        websocket: WebSocket соединение
        file_path: Путь к файлу на диске
        target_filename: Имя файла на сервере (если None, используется имя исходного файла)
    """
    path = Path(file_path)
    
    if not path.exists():
        raise FileNotFoundError(f"Файл не найден: {file_path}")
    
    filename = target_filename or path.name
    
    # Определяем, текстовый это файл или бинарный
    try:
        with open(path, 'r', encoding='utf-8') as f:
            content = f.read()
        # Текстовый файл - отправляем как есть
        upload_request = {
            "type": "upload_file",
            "filename": filename,
            "content": content
        }
    except UnicodeDecodeError:
        # Бинарный файл - кодируем в base64
        with open(path, 'rb') as f:
            binary_data = f.read()
        base64_data = base64.b64encode(binary_data).decode('utf-8')
        upload_request = {
            "type": "upload_file",
            "filename": filename,
            "content": f"base64:{base64_data}"
        }
    
    return upload_request

async def upload_local_file_example():
    """
    Пример загрузки локального файла с диска
    """
    uri = "ws://127.0.0.1:8899"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("✅ Подключено к серверу")
            print()
            
            # Пример: загружаем файл из текущей директории
            # Замените на путь к вашему файлу
            local_file = "example.txt"
            
            if os.path.exists(local_file):
                print(f"📤 Загрузка локального файла: {local_file}")
                upload_request = upload_file_from_disk(websocket, local_file)
                
                await websocket.send(json.dumps(upload_request))
                response = await websocket.recv()
                result = json.loads(response)
                
                print(f"📥 Получен ответ:")
                print(f"  Success: {result['success']}")
                print(f"  Message: {result.get('message', '')}")
                if result.get('error'):
                    print(f"  Error: {result['error']}")
            else:
                print(f"⚠️  Файл {local_file} не найден")
                print("💡 Создайте файл example.txt для тестирования")
            
    except Exception as e:
        print(f"❌ Ошибка: {e}")

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "--upload-local":
        # Режим загрузки локального файла
        asyncio.run(upload_local_file_example())
    else:
        # Обычный режим тестирования
        asyncio.run(test_file_upload())


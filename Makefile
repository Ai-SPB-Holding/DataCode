# Makefile для DataCode
# Удобные команды для сборки, тестирования и установки DataCode

.PHONY: help build test run install uninstall clean dev release examples

# Цель по умолчанию
help:
	@echo "🧠 DataCode - Доступные команды"
	@echo "================================"
	@echo ""
	@echo "Разработка:"
	@echo "  make build      - Собрать DataCode в режиме отладки"
	@echo "  make test       - Запустить все тесты"
	@echo "  make run        - Запустить DataCode REPL"
	@echo "  make dev        - Собрать и запустить в режиме разработки"
	@echo ""
	@echo "Релиз:"
	@echo "  make release    - Собрать DataCode в релизном режиме"
	@echo "  make install    - Установить DataCode как глобальную команду"
	@echo "  make uninstall  - Удалить глобальную команду DataCode"
	@echo ""
	@echo "Примеры:"
	@echo "  make examples      - Запустить все файлы примеров"
	@echo "  make run-example   - Запустить конкретный пример (FILE=path/to/file.dc)"
	@echo ""
	@echo "Тестирование:"
	@echo "  make test-cli   - Протестировать командную строку"
	@echo ""
	@echo "Обслуживание:"
	@echo "  make clean      - Очистить артефакты сборки"
	@echo ""
	@echo "Использование после установки:"
	@echo "  datacode                 # Запустить интерактивный REPL"
	@echo "  datacode filename.dc     # Выполнить файл filename.dc"
	@echo "  datacode --help          # Показать справку"
	@echo "  datacode --version       # Показать версию"
	@echo ""
	@echo "Примеры использования:"
	@echo "  datacode hello.dc                                    # Выполнить файл"
	@echo "  datacode examples/01-основы/hello.dc                # Выполнить пример"
	@echo "  datacode examples/01-основы/variables.dc            # Работа с переменными"
	@echo "  datacode examples/02-синтаксис/conditionals.dc     # Условные операторы"
	@echo "  datacode examples/04-функции/simple_functions.dc    # Функции"
	@echo "  datacode examples/05-циклы/for_loops.dc             # Циклы"

# Сборка в режиме отладки
build:
	@echo "🔨 Сборка DataCode (режим отладки)..."
	cargo build

# Сборка в релизном режиме
release:
	@echo "🔨 Сборка DataCode (релизный режим)..."
	cargo build --release

# Запуск тестов
test:
	@echo "🧪 Запуск тестов..."
	cargo test

# Запуск тестов с тихим выводом
test-quiet:
	@echo "🧪 Запуск тестов (тихий режим)..."
	cargo test --quiet

# Запуск тестов по категориям
test-language:
	@echo "🧪 Запуск тестов языковых возможностей..."
	cargo test language_features

test-data:
	@echo "🧪 Запуск тестов типов данных..."
	cargo test data_types

test-builtins:
	@echo "🧪 Запуск тестов встроенных функций..."
	cargo test builtins

test-errors:
	@echo "🧪 Запуск тестов обработки ошибок..."
	cargo test error_handling

test-performance:
	@echo "🧪 Запуск тестов производительности..."
	cargo test performance

test-integration:
	@echo "🧪 Запуск интеграционных тестов..."
	cargo test integration

# Запуск REPL
run:
	@echo "🚀 Запуск DataCode REPL..."
	cargo run

# Режим разработки (сборка + запуск)
dev: build run

# Установка как глобальная команда
install:
	@echo "📦 Глобальная установка DataCode..."
	@chmod +x install.sh
	@./install.sh

# Удаление глобальной команды
uninstall:
	@echo "🗑️  Удаление DataCode..."
	@chmod +x uninstall.sh
	@./uninstall.sh

# Запуск файлов примеров
examples:
	@echo "📚 Запуск примеров DataCode..."
	@echo ""
	@echo "🔹 Запуск hello.dc:"
	@cargo run --bin datacode -- examples/01-основы/hello.dc || cargo run -- examples/01-основы/hello.dc
	@echo ""
	@echo "🔹 Запуск variables.dc:"
	@cargo run --bin datacode -- examples/01-основы/variables.dc || cargo run -- examples/01-основы/variables.dc
	@echo ""
	@echo "🔹 Запуск showcase.dc:"
	@cargo run --bin datacode -- examples/06-демонстрации/showcase.dc || cargo run -- examples/06-демонстрации/showcase.dc

# Запуск конкретного примера
run-example:
	@if [ -z "$(FILE)" ]; then \
		echo "❌ Укажите файл: make run-example FILE=examples/01-основы/hello.dc"; \
	else \
		echo "🚀 Запуск $(FILE)..."; \
		cargo run --bin datacode -- $(FILE) || cargo run -- $(FILE); \
	fi

# Тестирование командной строки
test-cli: build
	@echo "🧪 Тестирование командной строки..."
	@echo ""
	@echo "🔹 Проверка --help:"
	@./target/debug/datacode --help | head -5
	@echo ""
	@echo "🔹 Проверка --version:"
	@./target/debug/datacode --version
	@echo ""
	@echo "✅ Командная строка работает корректно!"

# Очистка артефактов сборки
clean:
	@echo "🧹 Очистка артефактов сборки..."
	cargo clean

# Проверка форматирования и линтинга кода
check:
	@echo "🔍 Проверка кода..."
	cargo check
	cargo clippy
	cargo fmt --check

# Форматирование кода
format:
	@echo "✨ Форматирование кода..."
	cargo fmt

# Показать информацию о проекте
info:
	@echo "🧠 Информация о проекте DataCode"
	@echo "==============================="
	@echo "Название: ДатаКод"
	@echo "Версия: $(shell grep '^version' Cargo.toml | cut -d'"' -f2)"
	@echo "Язык: Rust"
	@echo "Лицензия: MIT"
	@echo ""
	@echo "📁 Структура проекта:"
	@echo "  src/           - Исходный код"
	@echo "  examples/      - Примеры .dc файлов"
	@echo "  tests/         - Тестовые файлы"
	@echo ""
	@echo "🔧 Доступные цели: build, test, run, install, examples"

FROM rust:latest

WORKDIR /usr/src/app

# Копируем только Cargo.toml и Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Подкладываем временный main.rs, чтобы cargo fetch не падал
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Кэшируем зависимости
RUN cargo fetch

# Теперь копируем остальной код поверх
COPY . .

# Сборка
RUN cargo build --release

CMD ["./target/release/DataCode"]
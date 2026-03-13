FROM rust:1.85

WORKDIR /usr/src/todo
COPY . .

RUN cargo install --path .

CMD ["todo"]

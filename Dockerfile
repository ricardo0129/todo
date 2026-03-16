FROM rust:1.85

WORKDIR /usr/src/todo
COPY . .

EXPOSE 8080
RUN cargo install --path .

CMD ["todo"]

FROM rust:1.75-buster as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cd http && cargo install --path .

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/http /usr/local/bin/http
CMD ["http"]

FROM ubuntu:latest

RUN mkdir -p /app
WORKDIR /app

ADD target/x86_64-unknown-linux-musl/release/surfjudge-actix /app
ADD static /app/static
ADD templates /app/templates
ADD config /app/config

CMD ["./surfjudge-actix"]

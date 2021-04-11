FROM rust:latest as planner
WORKDIR app

RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM rust:latest as cacher
WORKDIR app

RUN apt-get update \
  && apt-get install -y libzmq3-dev \
  && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --features zmq-receiver,zmq-notifier --recipe-path recipe.json


FROM rust:latest as builder
WORKDIR app

RUN apt-get update \
  && apt-get install -y libzmq3-dev \
  && rm -rf /var/lib/apt/lists/*

COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release --features zmq-receiver,zmq-notifier


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
  && apt-get install -y libzmq3-dev \
  && rm -rf /var/lib/apt/lists/*

ENV APP_USER=appuser
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}

ADD static ${APP}/static
ADD templates ${APP}/templates
ADD config ${APP}/config

COPY --from=builder /app/target/release/surfjudge-actix ${APP}/surfjudge-actix

CMD ["./surfjudge-actix"]

FROM rust:latest as builder

RUN USER=root cargo new --bin surfjudge-actix
WORKDIR ./surfjudge-actix

COPY ./Cargo.toml   ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/surfjudge_actix*
RUN cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app

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

COPY --from=builder /surfjudge-actix/target/release/surfjudge-actix ${APP}/surfjudge-actix

CMD ["./surfjudge-actix"]

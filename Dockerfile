FROM rust:alpine as builder
WORKDIR /usr/src/vote_bot
COPY . .
RUN apk add --no-cache musl-dev && cargo install --path .

FROM alpine
COPY --from=builder /usr/local/cargo/bin/vote_bot /bin/vote_bot
RUN mkdir /data
WORKDIR /data
CMD vote_bot

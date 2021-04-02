FROM rust:alpine as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine
COPY --from=builder /app/release/rick-roll /app
CMD ["./rick-roll"]
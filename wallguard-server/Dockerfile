FROM rust

WORKDIR /WallmonMonitor

COPY . /WallmonMonitor

EXPOSE 50051

RUN apt-get clean
RUN apt-get update
RUN apt-get install -y cmake

RUN cargo build --release

CMD ["./target/release/wallmon-monitor"]
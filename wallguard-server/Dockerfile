FROM rust

WORKDIR /wallguard-server

COPY . /wallguard-server

EXPOSE 50051

RUN apt-get clean
RUN apt-get update
RUN apt-get install -y cmake

RUN cargo build --release

CMD ["./target/release/wallguard-server"]
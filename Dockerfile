FROM ubuntu:latest

RUN apt update
RUN apt install -y curl

ADD . /src
RUN cd /src
RUN curl https://sh.rustup.rs -sSf | sh
RUN cargo build --release

CMD ./target/release/back_code
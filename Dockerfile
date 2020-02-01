FROM ubuntu:latest

ADD . /src
RUN cd /src
RUN curl https://sh.rustup.rs -sSf | sh
RUN cargo build --release

CMD ./target/release/back_code
FROM ubuntu:latest

RUN apt update
RUN apt install -y curl source

ADD . /src
RUN curl https://sh.rustup.rs -sSf --output installer
RUN sh installer -y
RUN source $HOME/.cargo/env && cd /src && cargo build --release

CMD /src/target/release/back_code
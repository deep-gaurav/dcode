FROM ubuntu:latest

RUN apt update
RUN apt install -y curl build-essential

ADD . /src
RUN curl https://sh.rustup.rs -sSf --output installer
RUN sh installer -y
RUN export PATH="$PATH:$HOME/.cargo/bin" && cd /src && cargo build --release

CMD /src/target/release/back_code
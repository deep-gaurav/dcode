FROM kalilinux/kali-rolling

RUN apt update
RUN apt install -y curl build-essential

RUN useradd -rm -d /home/ubuntu -s /bin/bash -g root -G sudo -u 1000 ubuntu

ADD . /src
RUN curl https://sh.rustup.rs -sSf --output rustinstaller
RUN sh rustinstaller -y
RUN export PATH="$PATH:$HOME/.cargo/bin" && cd /src && cargo build --release

CMD /src/target/release/back_code
FROM ubuntu:latest

RUN apt update
RUN apt install -y curl build-essential python3 aria2 unrar unzip tree zip wget lynx vim qbittorrent-nox

RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -
RUN echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list

RUN apt update && apt install -y yarn

ADD . /src
RUN curl https://sh.rustup.rs -sSf --output rustinstaller
RUN sh rustinstaller -y
RUN export PATH="$PATH:$HOME/.cargo/bin" && cd /src && cargo build --release

CMD /src/target/release/back_code

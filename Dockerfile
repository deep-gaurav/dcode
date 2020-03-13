FROM ubuntu:latest

RUN apt update
RUN apt install -y curl build-essential python3 aria2 unrar unzip tree zip wget lynx vim qbittorrent-nox

RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -
RUN echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list

RUN curl -sL https://deb.nodesource.com/setup_13.x | bash -

RUN apt update && apt install -y yarn nodejs
RUN touch /.bashrc

RUN apt install -y nano python3-pip git

RUN cd / && git clone https://github.com/sourcegraph/javascript-typescript-langserver.git
RUN cd /javascript-typescript-langserver && npm install && npm run build

ADD . /src
RUN curl https://sh.rustup.rs -sSf --output rustinstaller
RUN sh rustinstaller -y
RUN export PATH="$PATH:$HOME/.cargo/bin" && cd /src && cargo build --release

CMD cd /src && ./target/release/back_code

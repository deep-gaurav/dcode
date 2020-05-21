FROM ubuntu:latest

ARG DEBIAN_FRONTEND=noninteractive

RUN apt update
RUN apt install -y curl build-essential python3 aria2 unrar unzip tree zip wget lynx vim qbittorrent-nox wget

RUN apt install -y apt-transport-https software-properties-common
RUN wget -q https://xpra.org/gpg.asc -O- | apt-key add -

RUN add-apt-repository "deb https://xpra.org/ focal main"
RUN apt update

RUN apt install xpra xpra-html5 xterm

RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -
RUN echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list

RUN curl -sL https://deb.nodesource.com/setup_13.x | bash -

RUN apt update && apt install -y yarn nodejs
RUN touch /.bashrc

RUN apt install -y nano python3-pip git ffmpeg

RUN cd / && git clone https://github.com/sourcegraph/javascript-typescript-langserver.git
RUN cd /javascript-typescript-langserver && npm install && npm run build


RUN cd / && git clone https://github.com/wylieconlon/jsonrpc-ws-proxy.git
ADD ./lang_servers/servers.yml /jsonrpc-ws-proxy/
RUN cd /jsonrpc-ws-proxy/ && npm install && npm run prepare

RUN pip3 install python-language-server
RUN pip3 install 'python-language-server[all]'

RUN npm install -g typescript-language-server

ADD . /src
ADD .bashrc /.bashrc
ADD install_frontend.sh /install_frontend.sh
RUN curl https://sh.rustup.rs -sSf --output rustinstaller
RUN sh rustinstaller -y
RUN export PATH="$PATH:$HOME/.cargo/bin" && cd /src && cargo build --release



CMD cd /src && ./target/release/back_code

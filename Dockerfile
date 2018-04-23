FROM frolvlad/alpine-rust

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install -vv

EXPOSE 80

CMD /root/.cargo/bin/http_server
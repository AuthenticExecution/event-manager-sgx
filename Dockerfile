FROM authexec/docker-sgx-tools:latest

COPY src src
COPY Cargo.toml .
RUN cargo install --debug --path .

FROM authexec/docker-sgx-base:latest
COPY --from=0 /usr/local/cargo/bin/event_manager /bin/event_manager

CMD ["event_manager"]

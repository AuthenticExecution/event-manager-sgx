FROM registry.rosetta.ericssondevops.com/gftl-er-5g-hosts/authentic-execution/images/edp-tools:latest

COPY src src
COPY Cargo.toml .
RUN cargo install --debug --path .

FROM registry.rosetta.ericssondevops.com/gftl-er-5g-hosts/authentic-execution/images/edp-sgx-base:latest
COPY --from=0 /usr/local/cargo/bin/event_manager /bin/event_manager

CMD ["event_manager"]

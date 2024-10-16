# Start from the official Rust 1.81.0 image
FROM rust:1.81.0

RUN rustup component add rustfmt

# Set the working directory inside the container
WORKDIR /usr/src/app

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y curl

# Copy the Cargo.toml and Cargo.lock files to the container (if you have them)
COPY Cargo.toml Cargo.lock ./

# Copy the source code into the container
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/app/target/release/incremental \
    --mount=type=cache,target=/usr/src/app/target/release/build \
    --mount=type=cache,target=/usr/src/app/target/release/deps \
    --mount=type=cache,target=/usr/src/app/target/release/.fingerprint \
    cargo build --release

EXPOSE ${WEB_DATA_SERVICE_PORT}
RUN chmod +x ./target/release/web-data-service
CMD ["./target/release/web-data-service"]

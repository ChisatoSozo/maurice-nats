# Start from the official Rust 1.81.0 image
FROM rust:1.81.0

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files to the container (if you have them)
COPY Cargo.toml Cargo.lock ./

#echo minimal src/main.rs
RUN mkdir -p src
RUN echo 'fn main() { println!("Hello, world!"); }' > src/main.rs

# This step will cache the dependencies so that Docker doesn't have to download them again if they haven't changed
RUN cargo fetch

# Copy the source code into the container
COPY . .

# Build the Rust application (replace 'release' with 'debug' if you prefer)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/usr/local/cargo/git \
  --mount=type=cache,target=/usr/src/app/target/release/incremental \
  --mount=type=cache,target=/usr/src/app/target/release/build \
  --mount=type=cache,target=/usr/src/app/target/release/deps \
  --mount=type=cache,target=/usr/src/app/target/release/.fingerprint \
  cargo build --release

RUN chmod +x ./target/release/nats-echo
CMD ./target/release/nats-echo

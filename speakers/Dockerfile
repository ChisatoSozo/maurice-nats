# Start from the official Rust 1.81.0 image
FROM rust:1.81.0

RUN rustup component add rustfmt

# Set the working directory inside the container
WORKDIR /usr/src/app

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y curl

RUN curl --output-dir /etc/apt/trusted.gpg.d -O https://apt.fruit.je/fruit.gpg

RUN apt-get update && \
    apt-get install -y alsa-utils libasound2-dev mpv && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* 

RUN alsactl init && alsactl store
RUN wget -q https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp
RUN chmod +x /usr/local/bin/yt-dlp

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
RUN cargo build --release

EXPOSE ${SPEAKERS_PORT}

CMD cargo run --release

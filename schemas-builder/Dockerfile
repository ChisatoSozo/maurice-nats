# Use Alpine Linux as the base image
FROM alpine:3.18

# Install system dependencies
RUN apk add --no-cache \
  bash \
  cmake \
  g++ \
  git \
  make \
  curl \
  unzip \
  python3 \
  py3-pip \
  rust \
  cargo \
  nodejs \
  npm \
  clang \
  llvm-dev \
  libffi-dev \
  musl-dev \
  openssl-dev

# Install Rustup separately (since apk rustup is not included in base Alpine repo)
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add rust and cargo binaries to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Set the working directory
WORKDIR /opt

# Clone the FlatBuffers repository from GitHub
RUN git clone https://github.com/google/flatbuffers.git --branch v23.5.26 --depth 1

# Build FlatBuffers from source
WORKDIR /opt/flatbuffers

# Create build directory and compile using CMake
RUN cmake -G "Unix Makefiles" && make -j$(nproc)

# Install the compiled FlatBuffers binaries
RUN make install

# Set default working directory for subsequent commands
WORKDIR /workspace

COPY schemas schemas

RUN set -x
RUN flatc --rust -o out -I schemas

CMD ["ls", "out"]

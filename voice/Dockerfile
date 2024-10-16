# Start from the official Rust 1.81.0 image
FROM pytorch/pytorch:2.4.1-cuda12.1-cudnn9-devel

# Set the working directory inside the container
WORKDIR /usr/src/app

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y curl alsa-utils libasound2-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* 

RUN alsactl init && alsactl store

COPY requirements.txt ./requirements.txt

RUN pip install -r requirements.txt

# Copy the source code into the container
COPY . .

CMD ["python src/main.py"]

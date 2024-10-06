#!/bin/bash

# Define the build context and Dockerfile path
DOCKERFILE_PATH="schemas-builder/Dockerfile"
BUILD_CONTEXT="schemas-builder"

# Build the Docker image from the Dockerfile
docker build -f "$DOCKERFILE_PATH" -t schemas-builder "$BUILD_CONTEXT"

# Run the container with a dummy command to keep it alive (using /bin/sh)
CONTAINER_ID=$(docker run -d schemas-builder /bin/sh -c "sleep 1")

# Check if the container started successfully
if [ -n "$CONTAINER_ID" ]; then
  echo "Container started with ID: $CONTAINER_ID"

  rm -rf /tmp/out

  # Copy the /workspace/out folder from the running container to the host
  docker cp "$CONTAINER_ID":/workspace/out /tmp

  echo "Files copied to /tmp/out"

  #Copy /tmp/out/ts to ./web-client/src/schemas
  rm -rf ./web-client/src/schemas
  mkdir -p ./web-client/src/schemas
  cp -r /tmp/out/ts/* ./web-client/src/schemas

  #Copy /tmp/out/rs to ./nats-echo/src/schemas
  rm -rf ./nats-echo/src/schemas
  mkdir -p ./nats-echo/src/schemas
  cp -r /tmp/out/rs/* ./nats-echo/src/schemas

  #Copy /tmp/out/rs to ./speakers/src/schemas
  rm -rf ./speakers/src/schemas
  mkdir -p ./speakers/src/schemas
  cp -r /tmp/out/rs/* ./speakers/src/schemas

  # Stop and remove the container
  docker stop "$CONTAINER_ID"
  docker rm "$CONTAINER_ID"
else
  echo "Error: failed to start the container."
fi

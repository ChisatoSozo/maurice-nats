#!/bin/bash

# Define the build context and Dockerfile path
DOCKERFILE_PATH="schemas-builder/Dockerfile"
BUILD_CONTEXT="schemas-builder"

# Build the Docker image from the Dockerfile
docker build -f "$DOCKERFILE_PATH" -t schemas-builder "$BUILD_CONTEXT"

# Run the container with a dummy command to keep it alive (using /bin/sh)
CONTAINER_ID=$(docker run -d schemas-builder /bin/sh -c "sleep 10")

# Check if the container started successfully
if [ -n "$CONTAINER_ID" ]; then
  echo "Container started with ID: $CONTAINER_ID"

  # Copy the /workspace/out folder from the running container to the host
  docker cp "$CONTAINER_ID":/workspace/out ./local_folder

  echo "Files copied to ./local_folder"

  # Stop and remove the container
  docker stop "$CONTAINER_ID"
  docker rm "$CONTAINER_ID"
else
  echo "Error: failed to start the container."
fi

# Use an official Node.js image from the Docker Hub
FROM node:18-alpine

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the package.json and yarn.lock files to the working directory
COPY package.json yarn.lock ./

# Install dependencies using Yarn
RUN yarn install --frozen-lockfile

# Expose the port that the application will run on
EXPOSE ${WEB_CLIENT_PORT}

COPY . .

# Command to run the application in development mode using `yarn dev`
# with the specified host and port
CMD yarn dev --host 0.0.0.0 --port ${WEB_CLIENT_PORT}

# Use an official Node.js image from the Docker Hub
FROM node:18-alpine

# Set the working directory inside the container
WORKDIR /app

# Copy the package.json and yarn.lock files to the working directory
COPY package.json yarn.lock ./

# Install dependencies using Yarn
RUN yarn install --frozen-lockfile

# Copy the rest of the application files
COPY . .

# Build the application
RUN yarn build

# Install serve globally for serving static files
RUN yarn global add serve

# Expose the port that the application will run on
EXPOSE ${WEB_CLIENT_PORT}

# Command to serve the built application using `serve`
# -s: Serve the 'build' directory (commonly the output for static apps like React)
# -l: Specify the listening address and port (host:port)
CMD serve -s dist -l tcp://0.0.0.0:${WEB_CLIENT_PORT}

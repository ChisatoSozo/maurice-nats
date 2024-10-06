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

# Expose the port that the application will run on (usually set in the app)
EXPOSE ${WEB_CLIENT_PORT}

# Specify the command to run the application using Yarn
CMD yarn dev --host 0.0.0.0 --port ${WEB_CLIENT_PORT}

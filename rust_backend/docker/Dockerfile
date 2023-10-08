# Use debian:stretch as base image
FROM debian:stretch

# Update the package list and install gcc
RUN apt-get update && apt-get install -y gcc g++

# Copy the source code to the Docker container
COPY ./code /src

# Compile the code
WORKDIR /src
RUN gcc -o /app/program *.c || g++ -o /app/program *.cpp
CMD ["/app/program"]

FROM debian:stable

# Update the package list and install gcc
RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc g++ && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create the destination directory for the compiled program
RUN mkdir -p /app

# Copy the source code to the Docker container
COPY ./code /src

# Compile the code
WORKDIR /src
RUN gcc -o /app/program *.c

# Set the entry point
CMD ["/app/program"]

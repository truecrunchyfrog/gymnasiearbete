# Use a minimal Alpine Linux base image
FROM alpine:latest

# Install required packages
RUN apk --no-cache add \
    bash \
    coreutils \
    procps \
    perf \
    smem

# Set the working directory
WORKDIR /app

# Define the entrypoint with the command
ENTRYPOINT ["/bin/bash", "-c", "{ { time -f '{\"real\":%e,\"user\":%U,\"sys\":%S}' ./program.o ; } 2>&1 && perf stat -o perf_stats.json -e cpu-clock ./program.o ; } | smem -r -k -c 'pid pss' | awk 'NR==2{print \"\\\"peak_memory\\\":\" $3 }'"]

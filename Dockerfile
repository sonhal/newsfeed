# Stage 1: Build
FROM rust:1.84-slim as builder

# Install dependencies for building the project
RUN apt-get update && apt-get install -y libssl-dev pkg-config libc6

# Create and set the working directory
WORKDIR /app

# Copy only the necessary files to leverage caching
COPY Cargo.toml Cargo.lock ./

# Fetch dependencies to cache them
RUN cargo fetch

# Copy the rest of the code
COPY . .

# Build the release version
RUN cargo build --release

# Strip debug symbols to reduce binary size
RUN strip /app/target/release/newsfeed

# Stage 2: Final image
FROM debian:bookworm-slim

# Create a non-root user
RUN useradd -m rustuser

# Set working directory
WORKDIR /app

# Copy the compiled binary from the builder
COPY --from=builder /app/target/release/newsfeed .

# Change ownership to non-root user
RUN chown rustuser:rustuser /app/newsfeed

# Set the user
USER rustuser

# Expose the port (adjust if necessary)
EXPOSE 8080
# Set the entry point
ENTRYPOINT ["./newsfeed"]
CMD []
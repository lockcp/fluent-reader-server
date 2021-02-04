FROM debian:bullseye-slim
WORKDIR /app
ADD target/release .
CMD ["/app/fluent-reader-server"]
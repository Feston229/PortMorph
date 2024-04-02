# Port Morph (port_morph)

Port Morph is a versatile networking tool designed to facilitate secure and efficient communication across different network protocols. It leverages modern Rust programming practices to ensure high performance and reliability.

## Features

- **Flexible Configuration**: Easily configure the tool using a TOML file, allowing for quick adjustments to network settings and behaviors.
- **Secure Communication**: Utilizes [rustls](file:///home/gera/Projects/Rust/PortMorph/Cargo.toml#10%2C1-10%2C1) for TLS support, ensuring encrypted and secure data transmission.
- **Asynchronous Support**: Built on top of [tokio](file:///home/gera/Projects/Rust/PortMorph/Cargo.toml#13%2C1-13%2C1), Port Morph handles network operations asynchronously, improving scalability and responsiveness.
- **Extensible**: With optional features like [axum](file:///home/gera/Projects/Rust/PortMorph/Cargo.toml#8%2C1-8%2C1) and [reqwest](file:///home/gera/Projects/Rust/PortMorph/Cargo.toml#9%2C1-9%2C1), the tool can be extended to support web server capabilities and HTTP client functionalities.

## Getting Started

### Prerequisites

- Rust 1.35.1 or higher
- Cargo (Rust's package manager)

### Installation

1. Clone the repository:
   ```bash
   git clone https://example.com/port_morph.git
   ```
2. Navigate to the project directory:
   ```bash
   cd port_morph
   ```
3. Build the project using Cargo:
   ```bash
   cargo build --release
   ```

### Configuration

Port Morph is configured through a TOML file (`ptm.toml`). A default configuration file is provided, but you can specify a custom path by setting an environment variable `PTM_CONFIG_PATH`.

Example `ptm.toml`:
```toml
[server]
listen = "127.0.0.1:8080"
ssl = true
```

### Running

To start Port Morph, simply run the compiled binary:

```bash
./target/release/port_morph
```

To run Port Morph inside a Docker container, follow these steps:

1. Build the Docker image:
   ```bash
   docker build -t port_morph .
   ```
2. Run the container:
   ```bash
   docker run -d -p 8080:8080 --name port_morph_instance port_morph
   ```

This will start Port Morph in a detached mode, listening on port 8080 of your host machine.

Make sure to adjust the port settings in your `ptm.toml` and Docker run command as needed to match your desired configuration.

To simplify the deployment process, you can also use Docker Compose to run Port Morph along with its dependencies. Ensure you have `docker-compose` installed on your system.

1. Create a `docker-compose.yml` file in the project directory with the following content:
   ```yaml
   version: '3'
   services:
     port_morph:
       build: .
       ports:
         - "8080:8080"
       volumes:
         - .:/app
       environment:
         - PTM_CONFIG_PATH=/app/ptm.toml
   ```

2. Start the service using Docker Compose:
   ```bash
   docker-compose up -d
   ```

This will build and start Port Morph in a detached mode, similar to the Docker run command, but with the added benefits of Docker Compose for managing multi-container Docker applications.


## Contributing

Contributions are welcome! Please refer to the project's issues and pull requests to see what features or bugs are currently being worked on.

## License

Port Morph is licensed under the GNU General Public License v3.0. For more details, see the [LICENSE](LICENSE) file.

## Acknowledgments

- Rust Programming Language and its vibrant community.
- Open-source projects that inspired and contributed to the development of Port Morph.

For more information on how to use Port Morph and its features, please refer to the detailed documentation provided in the project's repository.
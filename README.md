# Port Morph

Port Morph is a versatile networking tool designed to facilitate secure and efficient communication across different network protocols. It leverages modern Rust programming practices to ensure high performance and reliability.

## Features

- **Flexible Configuration**: Easily configure the tool using a TOML file, allowing for quick adjustments to network settings and behaviors.
- **Secure Communication**: Utilizes [rustls](file:///home/gera/Projects/Rust/PortMorph/Cargo.toml#10%2C1-10%2C1) for TLS support, ensuring encrypted and secure data transmission.
- **Asynchronous Support**: Built on top of [tokio](file:///home/gera/Projects/Rust/PortMorph/Cargo.toml#13%2C1-13%2C1), Port Morph handles network operations asynchronously, improving scalability and responsiveness.

## Getting Started

### Prerequisites

- Rust 1.35.1 or higher
- Cargo (Rust's package manager)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/Feston229/PortMorph.git
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
```

To enable TLS encryption, specify the `ssl`, `cert_path`, and `key_path` under the `[server]` section in your `ptm.toml`. Ensure your certificate and key files are correctly referenced.
Example with tls:
```toml
[server]
listen = "127.0.0.1:8080"
ssl = true
cert_path = "/etc/ptm/cert.pem"
key_path = "/etc/ptm/key.pem"
```

Adding locations
```toml
[[location]]
name = "web"
path = "/"
forward_to = "127.0.0.1:3000"

[[location]]
name = "ssh"
path = "/ssh"
forward_to = "127.0.0.1:40000"

[[location]]
name = "api"
path = "/api"
forward_to = "127.0.0.1:5000"
```

### Running

To start Port Morph, simply run the compiled binary:

```bash
./target/release/port_morph
```

To deploy Port Morph using Docker, proceed with the following steps, utilizing the pre-built Dockerfile:

1. Build the Docker image using the provided pre_build.dockerfile:
   ```bash
   docker build -f dockerfiles/pre_build.dockerfile -t port_morph .
   ```
2. To run the container, ensure to replace `ptm.toml` with the path to your configuration file:
   ```bash
   docker run -d -p 8080:8080 --name port_morph_instance -v $(pwd)/ptm.toml:/etc/ptm/ptm.toml port_morph
   ```
3. For configurations utilizing SSL, attach your certificate and key file as follows:
   ```bash
   docker run -d -p 8080:8080 --name port_morph_instance -v $(pwd)/ptm.toml:/etc/ptm/ptm.toml -v $(pwd)/cert.pem:/etc/ptm/cert.pem -v $(pwd)/key.pem:/etc/ptm/key.pem port_morph
   ```

This will start Port Morph in a detached mode, listening on port 8080 of your host machine.

Make sure to adjust the port settings in your `ptm.toml` and Docker run command as needed to match your desired configuration.

To simplify the deployment process, you can also use Docker Compose to run Port Morph along with its dependencies. Ensure you have `docker-compose` installed on your system.

1. Create a `docker-compose.yml` file in the project directory with the following content:
   ```yaml
   version: "3.8"

   services:
     web:
       image: feston229/port_morph:latest
       ports:
         - "8080:8080"
       volumes:
         - ./ptm.toml:/etc/ptm/ptm.toml:ro # Config path
         - ./cert.pem:/etc/ptm/cert.pem:ro # cert in ssl setup
         - ./key.pem:/etc/ptm/key.pem:ro # key in ssl setup
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

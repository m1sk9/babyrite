# Installation

<!-- toc -->

## Using Docker Image

1. Pull the Docker Image from ghcr.io:

    ```sh
    # Latest version
    docker pull ghcr.io/m1sk9/babyrite:latest

    # Minor version (v0.x) - recommended
    docker pull ghcr.io/m1sk9/babyrite:v0

    # Specific version
    docker pull ghcr.io/m1sk9/babyrite:vX.Y.Z
    ```

2. Start babyrite by providing environment variables for the Docker Image you have obtained.

   Key whose `Required` column is `Yes` must be specified. If not specified, babyrite will not start correctly.

   | Key                 | Description                             | Required | Default |
   |---------------------|-----------------------------------------|----------|---------|
   | `CONFIG_FILE_PATH`  | Configuration file path (relative path) | No       | ---     |
   | `DISCORD_API_TOKEN` | Discord API Token                       | Yes      | ---     |

3. Copy configuration files:

    ```sh
    mkdir -p config
    curl -o config/config.yaml https://raw.githubusercontent.com/m1sk9/babyrite/main/config/config.yaml
    ```

4. Start babyrite:

    ```sh
    docker run -e CONFIG_FILE_PATH=/path/to/your/config.yaml -e DISCORD_API_TOKEN=your_discord_api_token --volume ./config/config.yaml:/config/config.yaml ghcr.io/m1sk9/babyrite
    ```

## Using Docker Compose

1. Create a `compose.yaml` file:

    ```yml
    services:
      app:
         image: ghcr.io/m1sk9/babyrite:v0
      env_file:
         - .env
      volumes:
         - ./config/config.yaml:/config/config.yaml
      environment:
         - RUST_LOG=babyrite=info
      restart: always
    ```

2. Create a `.env` file:

    ```sh
    CONFIG_FILE_PATH=/path/to/your/config.yaml
    DISCORD_API_TOKEN=your_discord_api_token
    ```

3. Copy configuration files:

    ```sh
    mkdir -p config
    curl -o config/config.yaml https://raw.githubusercontent.com/m1sk9/babyrite/main/config/config.yaml
    ```

4. Start babyrite:

    ```sh
    docker-compose up -d
    ```

## Using Binary (Build from Source)

> [!WARNING]
>
> Since babyrite is intended for use in a Docker environment, it is not distributed in binary form.
>
> If you want to use it in binary format, you need to compile it yourself. To compile it, you need to have [Rust](https://www.rust-lang.org/tools/install) installed.

1. Clone the repository:

    ```sh
    git clone https://github.com/m1sk9/babyrite.git
    ```

2. Build babyrite:

    ```sh
    cd babyrite
    cargo build --release
    ```

3. Copy configuration files:

    ```sh
    mkdir -p config
    curl -o config/config.yaml https://raw.githubusercontent.com/m1sk9/babyrite/main/config/config.yaml
    ```

4. Start babyrite:

    ```sh
    docker-compose up -d
    ```

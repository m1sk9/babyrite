# Installation

## Docker Image

Docker Image of babyrite can be obtained at ghcr.io:

```sh
# Latest version
docker pull ghcr.io/m1sk9/babyrite:latest

# Minor version (v0.x) - recommended
docker pull ghcr.io/m1sk9/babyrite:v0

# Specific version
docker pull ghcr.io/m1sk9/babyrite:vX.Y.Z
```

You can start babyrite by providing environment variables for the Docker Image you have obtained.

Key whose `Required` column is `Yes` must be specified. If not specified, babyrite will not start correctly.

| Key | Description | Required | Default |
| --- | ----------- | -------- | ------- |
| `DISCORD_API_TOKEN` | Discord API Token | Yes | --- |

## Build from Source

1. Clone the repository:

    ```sh
    git clone https://github.com/m1sk9/babyrite.git
    ```

2. Build babyrite:

    ```sh
    cd babyrite
    cargo build --release
    ```

3. Start babyrite:

    ```sh
    DISCORD_API_TOKEN=your_discord_api_token target/release/babyrite
    ```

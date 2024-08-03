# Setup

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

    | Key | Description | Required | Default |
    | --- | ----------- | -------- | ------- |
    | `DISCORD_API_TOKEN` | Discord API Token | Yes | --- |

3. Start babyrite:

    ```sh
    docker run -e DISCORD_API_TOKEN=your_discord_api_token ghcr.io/m1sk9/babyrite
    ```

## Using Docker Compose

1. Create a `compose.yaml` file:

    ```yml
    services:
      app:
        image: ghcr.io/m1sk9/babyrite:v0
        env_file:
          - .env
        restart: always
    ```

2. Create a `.env` file:

    ```sh
    DISCORD_API_TOKEN=your_discord_api_token
    ```

3. Start babyrite:

    ```sh
    docker-compose up -d
    ```

## Using Binary

[Learn more about using in binary](./installation.md#build-from-source).

> [!WARNING]
>
> Since babyrite is intended for use in a Docker environment, it is not distributed in binary form.
>
> If you want to use it in binary format, you need to compile it yourself. To compile it, you need to have [Rust](https://www.rust-lang.org/tools/install) installed.

## Channel Cache

> [!NOTE]
>
> Channel cache is available from [v0.3.0](https://github.com/m1sk9/babyrite/releases/tag/v0.3.0) ([v0.3.1](https://github.com/m1sk9/babyrite/releases/tag/v0.3.1)) onwards.

babyrite saves the channels of quoted messages as a cache. When there is another request to quote a message from the same channel, it uses the cached channel information to fetch the message.

By using the cache, babyrite does not use the Discord API to fetch the channel, making the message quoting faster.

If there is no cached channel, it fetches the channel using the Discord API (the method before v0.2.1). If there are many channels, quoting might take some time.

### Cache Expiration

If a cached entry is not accessed for 30 minutes, the cache is automatically cleared.

> [!IMPORTANT]
>
> To forcefully clear the cache, restart the active babyrite process.(For Docker Image, you can restart with docker restart.)
>
> However, this operation should generally not be necessary.

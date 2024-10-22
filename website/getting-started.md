---
title: Getting Started
editLink: true
---

# Getting Started

> [!NOTE]
>
> Prerequisites:
>
> - [Docker](https://docs.docker.com/get-docker/) and
>   [Docker Compose](https://docs.docker.com/compose/install/) must be
>   installed.
> - A Bot must be created in the
>   [Discord Developer Portal](https://discord.com/developers/applications) and
>   a token must be obtained.

> [!WARNING]
>
> babyrite requires **Message Content Intent**.
>
> Please enable `Message Content Intent` in the `Privileged Gateway Intents`
> section of the `Bot` tab.

1. Create a `.env` file in the directory and set the environment variables.
   - You can get a template for the `.env` file using the `curl` command.

   ```sh
   curl https://raw.githubusercontent.com/m1sk9/babyrite/main/.env.example -o .env
   ```

2. Create a `compose.yaml` file in the directory.

   ```yaml
   services:
     app:
       image: ghcr.io/m1sk9/babyrite:v1
       env_file:
         - .env
       restart: always
   ```

3. Run the command to start babyrite.

   ```sh
   docker compose up -d
   ```

After setup, refer to the following tutorials.

- [Customization (Settings)](./settings.md)
- [Usage](./guide/preview.md)

## Using the Test Version

Since babyrite does not distribute the test version as a Docker Image, you need
to compile it in your environment. Note that this method requires
[Rust](https://www.rust-lang.org/tools/install) to be installed.

> [!IMPORTANT]
>
> The test version may have unexpected bugs and the specifications may change
> without notice.
>
> Use it at your own risk and please provide feedback.

1. Clone the babyrite repository.

   ```sh
   git clone https://github.com/m1sk9/babyrite.git
   ```

2. Move to the directory and build.

   ```sh
   cd babyrite
   cargo build --release
   ```

3. Create a `.env` file in the directory and set the environment variables.
4. Run the babyrite executable.

   ```sh
   ./target/release/babyrite
   ```

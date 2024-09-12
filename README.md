# Rust Client
An example rust client for the Versa protocol, with both sending and receiving capabilities (enabled by feature flag)

## Feature Flags
- `sender` - Enables the sending capabilities of the client
- `receiver` - Enables the receiving capabilities of the client

In production use, you would likely only enable one of these feature flags, depending on the role of the client.

## Usage

Run the client with the following command:
```sh
cargo run --features sender
```

## With Docker

Build the image providing the desired feature flag as a build argument:
```sh
docker build --build-arg features=sender .
```

Run the image with the necessary environment variables:
```sh
docker run \
    -e REGISTRY_URL=https://registry.versa.org \
    -e VERSA_CLIENT_ID=versa_cid_xxxxxxxxxxxxx \
    -e VERSA_CLIENT_SECRET=versa_csk_xxxxxxxxx \
    -p 8080:8080 \
    87c6faff1243
```

# Rust Client
An example rust client for the Versa protocol, with both sending and receiving capabilities (enabled by feature flag)

## Feature Flags
- `sender` - Enables the sending capabilities of the client
- `receiver` - Enables the receiving capabilities of the client

In production use, you would likely only enable one of these feature flags, depending on the role of the client.

## Usage

Run the client with the following command:
```sh
cargo run --features receiver
```
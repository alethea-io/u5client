# u5client

`u5client` is a command-line interface (CLI) tool designed for interacting with UTXO RPC APIs. It allows users to fetch specific blocks, dump block history, and subscribe to blockchain tip updates.

## Installation

To install `u5client`, ensure you have Rust and Cargo installed on your machine. Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/alethea-io/u5client.git
cd u5client
cargo build --release
```

The executable will be available under `target/release/u5client`.

## Configuration

Before using `u5client`, you need to set up a configuration file named `config.toml` by default, or you can specify another file name with the `--config` option when running commands. Here's an example configuration:

```toml
url = "http://localhost:50051"
save_dir = "/path/to/your/save/directory"
```

- `url`: URL of the UTXO RPC API server.
- `save_dir`: Directory where the blocks will be saved if the save option is enabled.

## Usage

`u5client` supports several commands:

### Fetch

Fetch blocks from the UTXO RPC API based on their references.

**Usage**:

```bash
u5client fetch --refs <block-ref1> <block-ref2> ... [--save]
```

- `--refs`: A list of block references to fetch, formatted as `index-hash`.
- `--save`: Optional flag. If set, fetched blocks will be saved as JSON files in the configured `save_dir`. If omitted, blocks will be printed to the console.

**Example**:

```bash
u5client fetch --refs 50028-8046dce8b943dc327c04712641b2db446860afafe4602e0d779bbc2bfc5b7fc2 50030-bae8938f06f9c6f7ba366688aac6dc649567d7022fb39e3190cee44952840ef3 --save
```

### Dump

Dump a series of blocks starting from a specified block reference.

**Usage**:

```bash
u5client dump --ref <block-ref> --num-blocks <number> [--save]
```

- `--ref`: The starting block reference, formatted as `index-hash`.
- `--num_blocks`: Number of blocks to dump starting from the reference.
- `--save`: Optional flag. If set, dumped blocks will be saved as JSON files. Otherwise, they will be printed to the console.

**Example**:

```bash
u5client dump --ref 50028-8046dce8b943dc327c04712641b2db446860afafe4602e0d779bbc2bfc5b7fc2 --num-blocks 10 --save
```

### Follow

Subscribe to updates at the tip of the blockchain, optionally starting from specified block references.

**Usage**:

```bash
u5client follow [--refs <block-ref1> <block-ref2> ...]
```

- `--refs`: Optional. Block references to establish intersection points for the follow operation.

**Example**:

```bash
u5client follow
```

Each command outputs results directly to the console unless the `--save` flag is used, which diverts output to JSON files in the specified save directory.

## Contributions

Contributions are welcome. Please fork the repository, make your changes, and submit a pull request.
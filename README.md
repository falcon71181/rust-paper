# Rust Paper

A Rust-based wallpaper manager for Linux/UNIX systems that fetches wallpapers from [Wallhaven](https://wallhaven.cc/).

## Installation

To get started with `rust-paper`, first install it:

```bash
cargo install rust-paper
```

## Configuration

Configuration files are stored in different locations depending on your operating system:

- **Linux:** `~/.config/rust-paper/config.toml`
- **macOS:** `~/Library/Application Support/rs.rust-paper/config.toml`

### Example `config.toml`

```toml
save_location = "/Users/abhaythakur/Pictures/wall"
integrity = true
```

- `save_location`: The directory where wallpapers will be saved.
- `integrity`: If set to `true`, SHA256 checksums will be used for integrity verification.

### Additional Files

- `wallpaper.lock`: This file is used for integrity checks when `integrity` is set to `true`.
- `wallpapers.lst`: This file stores the IDs of the wallpapers from Wallhaven. An example of its content is shown below:

```plaintext
p9pzk9
x6m3gl
gpl8d3
5gqmg7
qzp8dr
yx3kok
85pgqk
3lgk6y
kx6yqm
o5ww39
o5m9xm
l8rloq
l8o2op
7pmgv9
```

## Usage

Once configured, you can run the application to download and manage wallpapers seamlessly.

### Command Line Interface

```bash
rust-paper <COMMAND>
```

#### Commands:
- `sync`  Sync wallpapers
- `help`  Print this message or the help of the given subcommand(s)

#### Options:
- `-h, --help`  Print help

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

---

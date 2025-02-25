Hermes
======

Quickly send messages to a configured Telegram chat on the command line.

Installation
------------

### Cargo

```bash
$ cargo install --git=https://github.com/darkwater/hermes
```

### Nix Flake

```bash
$ nix profile install github:darkwater/hermes
```

Or, to try it out without installing:

```bash
$ export HERMES_TOKEN="..."
$ export HERMES_CHAT_ID="..."
$ nix run github:darkwater/hermes send "Hello, world!"
```

Configuration
-------------

While in early development, this readme will probably not be complete. See
`src/config.rs` for the full list of configuration options, and the exact
mechanism for loading them.

Put a `config.toml` in a config directory such as `~/.config/hermes/` or
`/etc/hermes/` with the following contents:

```toml
token = "1234567890:AAHpq9Lj5jGR2PXpbH5KG6RBrYp4WJaYJo5"
chat_id = 63987654
```

Usage
-----

```bash
# Send a message to the configured chat
$ hermes send "Hello, world!"

# Send an image to the configured chat
$ hermes send --image img.png

# Send multiple images to the configured chat
$ hermes send "Here's your daily pet pictures" --image $(ls ~/cat_pics | shuf -n 1) --image $(ls ~/dog_pics | shuf -n 1)

# Send a message with a button, and wait until someone presses it
$ hermes wait "It's Monday morning! Upgrade the server?" "Yes!" && apt upgrade

# Send a message with a button, and only wait up to the specified timeout
$ hermes wait "It's Monday morning! Upgrade the server? Answer quickly!" "Yes!" --timeout=60 && apt upgrade

# Send a message with multiple buttons, wait until someone presses it, and print the number of the pressed button
$ [[ $(hermes wait "It's Monday morning! Upgrade the server?" "Yes!" "No!") == "0" ]] && apt upgrade
```

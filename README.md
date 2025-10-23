# NostrElixir

A complete Elixir wrapper for the [nostr](https://github.com/rust-nostr/nostr) Rust library, built with Rustler for high performance.

## Modular API Structure

NostrElixir is organized into logical modules for clarity and maintainability:

- `NostrElixir.Keys`   – Key management (generation, parsing, conversions)
- `NostrElixir.Nip19`  – NIP-19 encoding/decoding
- `NostrElixir.Event`  – Event creation, signing, verification, helpers
- `NostrElixir.Filter` – Filter creation and helpers
 - `NostrElixir.Nip02`  – Contact list events (NIP-02)
 - `NostrElixir.Nip04`  – Encrypted DMs (NIP-04)
 - `NostrElixir.Nip06`  – HD wallet derivation (NIP-06)
 - `NostrElixir.Nip09`  – Deletion events (NIP-09)
 - `NostrElixir.Nip10`  – Threading and text note helpers (NIP-10)
 - `NostrElixir.Nip17`  – Private DMs (kind 4 events)
 - `NostrElixir.Nip19`  – NIP-19 encoding/decoding
 - `NostrElixir.Nip23`  – Long-form content (NIP-23)
 - `NostrElixir.Nip44`  – Encrypted DMs v2 (NIP-44)
 - `NostrElixir.Nip57`  – Zaps (NIP-57)
 - `NostrElixir.Nip65`  – Relay list metadata (NIP-65)
 - `NostrElixir.Mnemonic` – Mnemonic generation and seed conversion (helpers for NIP-06)

The root `NostrElixir` module provides a facade for common operations, but direct use of submodules is recommended for clarity and maintainability.

## Usage Examples

### Key Management
```elixir
alias NostrElixir.Keys

# Generate a new keypair
keys = Keys.generate_keypair()
# %{public_key: ..., secret_key: ..., npub: ..., nsec: ...}

# Parse a secret key
parsed = Keys.parse_keypair(keys.nsec)

# Convert keys
npub = Keys.public_key_to_bech32(parsed.public_key)
nsec = Keys.secret_key_to_bech32(parsed.secret_key)
```

### NIP-19 Encoding/Decoding
```elixir
alias NostrElixir.Nip19

npub = Nip19.encode("npub", "eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619")
result = Nip19.decode_map(npub)
# %{data_type: "npub", data: ...}
```

### NIP-04 Encrypted Direct Messages
```elixir
alias NostrElixir.Nip04
alias NostrElixir.Keys

sender = Keys.generate_keypair()
receiver = Keys.generate_keypair()

ciphertext = Nip04.encrypt(sender.secret_key, receiver.public_key, "hello")
plaintext = Nip04.decrypt(receiver.secret_key, sender.public_key, ciphertext)
# "hello"
```

### NIP-44 Encrypted Direct Messages v2
```elixir
alias NostrElixir.Nip44
alias NostrElixir.Keys

sender = Keys.generate_keypair()
receiver = Keys.generate_keypair()

ciphertext = Nip44.encrypt(sender.secret_key, receiver.public_key, "hello")
plaintext = Nip44.decrypt(receiver.secret_key, sender.public_key, ciphertext)
# "hello"
```

### Event Creation, Signing, and Verification
```elixir
alias NostrElixir.{Keys, Event}

keys = Keys.generate_keypair()
event_json = Event.new(keys.public_key, "Hello, Nostr!", 1, [])
signed_event_json = Event.sign(event_json, keys.secret_key)
verified = Event.verify(signed_event_json) # true
```

### Filter Creation and Helpers
```elixir
alias NostrElixir.Filter

# Create a filter for fetching text notes (kind 1) by a user
filter_json = Filter.user_notes_filter("eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619")
filter = Jason.decode!(filter_json)
# filter["authors"] == ["..."]

# Create a filter for fetching a user's follow list (kind 3)
follow_list_filter_json = Filter.user_follow_list_filter("eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619")
follow_list_filter = Jason.decode!(follow_list_filter_json)
# follow_list_filter["kinds"] == [3]

# Hashtag search
filter_json = Filter.search_hashtag("nostr")
```

## Backward Compatibility

The root `NostrElixir` module still provides delegates for the most common operations, so existing code will continue to work. However, for new code, it is recommended to use the submodules directly.

## Features

- Key Management: Generate, parse, and convert keys
- NIP-19: Encode/decode bech32 addresses
- Event Management: Create, sign, verify, and serialize events
- Filter Management: Create filters for querying events
- Idiomatic, modular Elixir API
- High performance via Rust NIFs

## Roadmap

- [ ] Async event signing support (if/when needed)
- [ ] Full event verification implementation (if nostr Rust API changes)
- [ ] Relay connection management
- [ ] Event subscription handling
- [ ] More convenience functions for common nostr operations

## License

MIT


# Changelog

## 0.45.4

Aligns the Hex package with rust-nostr `nostr` 0.45.4. This is a breaking release relative to Hex `0.2.0`.

### Added

- NIP-19 TLV encoding and decoding for `naddr`, `nevent`, and `nprofile` via `NostrElixir.Nip19.Address`

### Changed

- Bump rust-nostr `nostr` to 0.45.4 and keep the Hex version in sync
- `NostrElixir.Nip02.create_follow_list_event/2` and `NostrElixir.Nip65.create_relay_list_event/2` now take the author's secret key (or a keys map) and return a correctly signed event
- Hex description and README now document the Rust toolchain requirement
- `rustler` is a compile-time dependency (`runtime: false`)

### Removed

- NIP-57 private and anonymous zap helpers (removed upstream in `nostr` 0.45)

### Fixed

- NIP-02 and NIP-65 no longer sign events with a hardcoded test key

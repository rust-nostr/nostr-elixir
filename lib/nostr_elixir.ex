defmodule NostrElixir do
  @moduledoc """
  NostrElixir: Elixir wrapper for the nostr Rust library.

  ## Modules

    * `NostrElixir.Keys`   – Key management (generation, parsing, conversions)
    * `NostrElixir.Nip19`  – NIP-19 encoding/decoding
    * `NostrElixir.Event`  – Event creation, signing, verification, helpers
    * `NostrElixir.Filter` – Filter creation and helpers
    * `NostrElixir.Nip04`  – Encrypted DMs (NIP-04)
    * `NostrElixir.Nip44`  – Encrypted DMs v2 (NIP-44)

  See the documentation for each module for details and examples.
  """

  alias NostrElixir.Keys
  alias NostrElixir.Nip19
  alias NostrElixir.Event
  alias NostrElixir.Filter

  # Optionally, re-export the most common functions for backward compatibility
  defdelegate generate_keypair(), to: Keys
  defdelegate parse_keypair(secret_key), to: Keys
  defdelegate nip19_encode(type, data), to: Nip19, as: :encode
  defdelegate nip19_decode(bech32_string), to: Nip19, as: :decode
  defdelegate nip19_decode_map(bech32_string), to: Nip19, as: :decode_map
  defdelegate new_event(pubkey, content, kind, tags), to: Event, as: :new
  defdelegate sign_event(event_json, secret_key), to: Event, as: :sign
  defdelegate verify_event(event_json), to: Event, as: :verify
  defdelegate create_text_note(keys, content), to: Event
  defdelegate create_metadata(keys, metadata), to: Event
  defdelegate new_filter(filter_spec), to: Filter, as: :new
  defdelegate user_notes_filter(pubkey), to: Filter
  defdelegate user_follow_list_filter(pubkey), to: Filter
  defdelegate get_user_metadata(pubkey), to: Filter
  defdelegate search_hashtag(hashtag), to: Filter
  defdelegate recent_events(), to: Filter

  # Keep the NIF stubs for Rustler
  use Rustler, otp_app: :nostr_elixir, crate: :nostr_nif
  def keys_generate_nif, do: :erlang.nif_error(:nif_not_loaded)
  def keys_parse_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def keys_public_key_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def keys_secret_key_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def keys_public_key_bech32_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def keys_secret_key_bech32_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def keys_secret_key_hex_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def parser_parse_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def nip19_encode_nif(_, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip19_decode_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def event_new_nif(_, _, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def event_sign_nif(_, _), do: :erlang.nif_error(:nif_not_loaded)
  def event_verify_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def event_to_json_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def event_from_json_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def filter_new_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def filter_to_json_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def filter_from_json_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def nip06_generate_mnemonic_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def nip06_mnemonic_to_seed_nif(_, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip06_derive_key_nif(_, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip06_validate_mnemonic_nif(_), do: :erlang.nif_error(:nif_not_loaded)
  def nip04_encrypt_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip04_decrypt_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip44_encrypt_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip44_decrypt_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip57_private_zap_request_nif(_, _, _, _, _, _, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip57_anonymous_zap_request_nif(_, _, _, _, _, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip57_decrypt_sent_private_zap_message_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip57_decrypt_received_private_zap_message_nif(_, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip17_encrypt_dm_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip17_decrypt_dm_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def nip65_create_relay_list_event_nif(_relays, _pubkey), do: :erlang.nif_error(:nif_not_loaded)
  def nip65_extract_relay_list_nif(_event_json), do: :erlang.nif_error(:nif_not_loaded)
  def nip02_create_contact_list_event_nif(_contacts, _pubkey), do: :erlang.nif_error(:nif_not_loaded)
  def nip02_extract_contacts_nif(_event_json), do: :erlang.nif_error(:nif_not_loaded)
  def nip10_create_text_note_nif(_keys_json, _content), do: :erlang.nif_error(:nif_not_loaded)
  def nip10_create_text_note_reply_nif(_keys_json, _content, _reply_to_json, _root_json, _relay_url), do: :erlang.nif_error(:nif_not_loaded)


  # Parse text tokens (convenience wrapper)
  def parse_text_tokens(text) do
    tokens_json = parser_parse_nif(text)
    tokens = Jason.decode!(tokens_json)

    Enum.map(tokens, fn token ->
      %{
        token_type: token["token_type"],
        value: token["value"]
      }
    end)
  end
end

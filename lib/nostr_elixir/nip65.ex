defmodule NostrElixir.Nip65 do
  @moduledoc """
  NIP-65: Relay List Metadata

  This module provides functions to create and extract relay list metadata events as specified in [NIP-65](https://github.com/nostr-protocol/nips/blob/master/65.md).

  ## Examples

      iex> relays = [
      ...>   {"wss://relay1.example.com", "read"},
      ...>   {"wss://relay2.example.com", "write"},
      ...>   {"wss://relay3.example.com", nil}
      ...> ]
      iex> pubkey = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
      iex> event_json = NostrElixir.Nip65.create_relay_list_event(relays, pubkey)
      iex> is_binary(event_json)
      true
      iex> NostrElixir.Nip65.extract_relay_list(event_json)
      [
        {"wss://relay1.example.com", "read"},
        {"wss://relay2.example.com", "write"},
        {"wss://relay3.example.com", nil}
      ]

  The relay list is a list of `{relay_url, metadata}` tuples, where `metadata` is either "read", "write", or `nil`.
  The public key must be a 64-character hex string.
  """

  defmodule Relay do
    @moduledoc """
    Struct representing a relay entry for NIP-65 relay list.

    * `:url` - Relay URL (string)
    * `:metadata` - "read", "write", or nil
    """
    @enforce_keys [:url]
    defstruct [:url, :metadata]
    @type t :: %__MODULE__{url: String.t(), metadata: String.t() | nil}
  end

  @doc """
  Validate a relay URL (must start with ws:// or wss://).
  Returns true if valid, false otherwise.
  """
  @spec valid_relay_url?(String.t()) :: boolean
  def valid_relay_url?(url) when is_binary(url) do
    String.starts_with?(url, ["ws://", "wss://"]) and String.length(url) > 8
  end

  @doc """
  Validate relay metadata (must be "read", "write", or nil).
  Returns true if valid, false otherwise.
  """
  @spec valid_metadata?(String.t() | nil) :: boolean
  def valid_metadata?(nil), do: true
  def valid_metadata?(meta) when is_binary(meta), do: meta in ["read", "write"]
  def valid_metadata?(_), do: false

  @doc """
  Convert a `{url, metadata}` tuple to a %Relay{} struct.
  """
  @spec tuple_to_struct({String.t(), String.t() | nil}) :: Relay.t()
  def tuple_to_struct({url, meta}), do: %Relay{url: url, metadata: meta}

  @doc """
  Convert a %Relay{} struct to a `{url, metadata}` tuple.
  """
  @spec struct_to_tuple(Relay.t()) :: {String.t(), String.t() | nil}
  def struct_to_tuple(%Relay{url: url, metadata: meta}), do: {url, meta}

  @doc """
  Pretty-print a relay list (list of tuples or structs).
  """
  @spec pretty_print([Relay.t()] | [{String.t(), String.t() | nil}]) :: String.t()
  def pretty_print(list) when is_list(list) do
    list
    |> Enum.map(fn
      %Relay{url: url, metadata: meta} -> "- #{url}#{if meta, do: " (#{meta})", else: ""}"
      {url, meta} -> "- #{url}#{if meta, do: " (#{meta})", else: ""}"
    end)
    |> Enum.join("\n")
  end

  @doc """
  Create a relay list event from a list of `{relay_url, metadata}` tuples and a public key.

  - `relays`: List of `{relay_url, metadata}` tuples. `metadata` can be "read", "write", or `nil`.
  - `pubkey`: 64-character hex string public key.

  Returns the event as a JSON string.

  ## Example

      iex> relays = [{"wss://relay.example.com", "read"}]
      iex> pubkey = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
      iex> event_json = NostrElixir.Nip65.create_relay_list_event(relays, pubkey)
      iex> is_binary(event_json)
      true
  """
  @spec create_relay_list_event([{String.t(), String.t() | nil}], String.t()) :: String.t()
  def create_relay_list_event(relays, pubkey) when is_list(relays) and is_binary(pubkey) do
    NostrElixir.nip65_create_relay_list_event_nif(relays, pubkey)
  end

  @doc """
  Extract the relay list from an event JSON string.

  Returns a list of `{relay_url, metadata}` tuples.

  ## Example

      iex> event_json = NostrElixir.Nip65.create_relay_list_event([
      ...>   {"wss://relay.example.com", "read"}
      ...> ], "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
      iex> NostrElixir.Nip65.extract_relay_list(event_json)
      [{"wss://relay.example.com", "read"}]
  """
  @spec extract_relay_list(String.t()) :: [{String.t(), String.t() | nil}]
  def extract_relay_list(event_json) when is_binary(event_json) do
    NostrElixir.nip65_extract_relay_list_nif(event_json)
  end
end

defmodule NostrElixir.Nip65 do
  @moduledoc """
  NIP-65: Relay List Metadata

  This module provides functions to create and extract relay list metadata events as specified in [NIP-65](https://github.com/nostr-protocol/nips/blob/master/65.md).

  ## Examples

      keys = NostrElixir.Keys.generate_keypair()
      relays = [
        {"wss://relay1.example.com", "read"},
        {"wss://relay2.example.com", "write"},
        {"wss://relay3.example.com", nil}
      ]
      event_json = NostrElixir.Nip65.create_relay_list_event(relays, keys.secret_key)
      NostrElixir.Nip65.extract_relay_list(event_json)

  The relay list is a list of `{relay_url, metadata}` tuples, where `metadata` is either "read", "write", or `nil`.
  Events are signed with the author's secret key (hex, bech32, or a keys map).
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
  Create a signed relay list event from a list of `{relay_url, metadata}` tuples.

  - `relays`: List of `{relay_url, metadata}` tuples. `metadata` can be "read", "write", or `nil`.
  - `secret_key`: Author's secret key (hex or bech32), or a keys map with `:secret_key`.

  Returns the event as a JSON string.
  """
  @spec create_relay_list_event([{String.t(), String.t() | nil}], String.t() | map()) :: String.t()
  def create_relay_list_event(relays, %{secret_key: secret_key}) do
    create_relay_list_event(relays, secret_key)
  end

  def create_relay_list_event(relays, secret_key) when is_list(relays) and is_binary(secret_key) do
    case NostrElixir.nip65_create_relay_list_event_nif(relays, secret_key) do
      {:error, reason} -> raise ArgumentError, "NIP-65 create_relay_list_event failed: #{reason}"
      result -> result
    end
  end

  @doc """
  Extract the relay list from an event JSON string.

  Returns a list of `{relay_url, metadata}` tuples.
  """
  @spec extract_relay_list(String.t()) :: [{String.t(), String.t() | nil}]
  def extract_relay_list(event_json) when is_binary(event_json) do
    NostrElixir.nip65_extract_relay_list_nif(event_json)
  end
end

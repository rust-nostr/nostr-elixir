defmodule NostrElixir.Nip02 do
  @moduledoc """
  NIP-02: Follow List

  Functions to create and extract follow list events (kind 3).

  ## Examples

      keys = NostrElixir.Keys.generate_keypair()
      follows = [
        {followed_pubkey, "wss://relay.example.com", "Alice"},
        {other_pubkey, nil, nil}
      ]
      event_json = NostrElixir.Nip02.create_follow_list_event(follows, keys.secret_key)
      NostrElixir.Nip02.extract_follows(event_json)

  The term 'contact' is used in some libraries, but the protocol and this module use 'follow' to match NIP-02.
  """

  defmodule Follow do
    @moduledoc """
    Struct representing a follow entry for NIP-02 follow list.
    * `:pubkey` - Public key (hex or bech32 string)
    * `:relay_url` - Relay URL (string or nil)
    * `:alias` - Alias (string or nil)
    """
    @enforce_keys [:pubkey]
    defstruct [:pubkey, :relay_url, :alias]
    @type t :: %__MODULE__{
            pubkey: String.t(),
            relay_url: String.t() | nil,
            alias: String.t() | nil
          }
  end

  @doc """
  Create a signed follow list event from a list of `{pubkey, relay_url, alias}` tuples.

  The second argument is the author's secret key (hex or bech32) or a keys map
  with a `:secret_key` field. The event pubkey is derived from that key.
  Returns the event as a JSON string.
  """
  @spec create_follow_list_event(
          [{String.t(), String.t() | nil, String.t() | nil}],
          String.t() | map()
        ) :: String.t()
  def create_follow_list_event(follows, %{secret_key: secret_key}) do
    create_follow_list_event(follows, secret_key)
  end

  def create_follow_list_event(follows, secret_key) when is_list(follows) and is_binary(secret_key) do
    case NostrElixir.nip02_create_contact_list_event_nif(follows, secret_key) do
      {:error, reason} -> raise ArgumentError, "NIP-02 create_follow_list_event failed: #{reason}"
      result -> result
    end
  end

  @doc """
  Extract the follow list from an event JSON string.
  Returns a list of `{pubkey, relay_url, alias}` tuples.
  """
  @spec extract_follows(String.t()) :: [{String.t(), String.t() | nil, String.t() | nil}]
  def extract_follows(event_json) when is_binary(event_json) do
    NostrElixir.nip02_extract_contacts_nif(event_json)
  end

  @doc """
  Convert a `{pubkey, relay_url, alias}` tuple to a %Follow{} struct.
  """
  @spec tuple_to_struct({String.t(), String.t() | nil, String.t() | nil}) :: Follow.t()
  def tuple_to_struct({pk, url, a}), do: %Follow{pubkey: pk, relay_url: url, alias: a}

  @doc """
  Convert a %Follow{} struct to a `{pubkey, relay_url, alias}` tuple.
  """
  @spec struct_to_tuple(Follow.t()) :: {String.t(), String.t() | nil, String.t() | nil}
  def struct_to_tuple(%Follow{pubkey: pk, relay_url: url, alias: a}), do: {pk, url, a}

  @doc """
  Validate a relay URL (must start with ws:// or wss://).
  """
  @spec valid_relay_url?(String.t() | nil) :: boolean
  def valid_relay_url?(nil), do: true

  def valid_relay_url?(url) when is_binary(url),
    do: String.starts_with?(url, ["ws://", "wss://"]) and String.length(url) > 8

  def valid_relay_url?(_), do: false

  @doc """
  Validate a public key (hex or bech32, 64+ chars).
  """
  @spec valid_pubkey?(String.t()) :: boolean
  def valid_pubkey?(pk) when is_binary(pk), do: String.length(pk) >= 32
  def valid_pubkey?(_), do: false

  @doc """
  Pretty-print a follow list (list of tuples or structs).
  """
  @spec pretty_print([Follow.t()] | [{String.t(), String.t() | nil, String.t() | nil}]) :: String.t()
  def pretty_print(list) when is_list(list) do
    list
    |> Enum.map(fn
      %Follow{pubkey: pk, relay_url: url, alias: a} ->
        "- #{pk}#{if url, do: " [#{url}]", else: ""}#{if a, do: " (#{a})", else: ""}"

      {pk, url, a} ->
        "- #{pk}#{if url, do: " [#{url}]", else: ""}#{if a, do: " (#{a})", else: ""}"
    end)
    |> Enum.join("\n")
  end
end

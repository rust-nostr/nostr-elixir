defmodule NostrElixir.Nip19.Address do
  @moduledoc """
  NIP-19 TLV encoding and decoding for `naddr`, `nevent`, and `nprofile`.

  Backed by the rust-nostr NIF (same crate as the rest of NostrElixir).
  """

  @type naddr :: %{
          kind: integer(),
          pubkey: String.t(),
          identifier: String.t(),
          relays: [String.t()]
        }

  @type nevent :: %{
          event_id: String.t(),
          author: String.t() | nil,
          kind: integer() | nil,
          relays: [String.t()]
        }

  @type nprofile :: %{
          pubkey: String.t(),
          relays: [String.t()]
        }

  defdelegate nip19_encode_naddr_nif(kind, pubkey, identifier, relays), to: NostrElixir
  defdelegate nip19_encode_nevent_nif(event_id, author, relays), to: NostrElixir
  defdelegate nip19_encode_nprofile_nif(pubkey, relays), to: NostrElixir
  defdelegate nip19_decode_address_nif(encoded), to: NostrElixir

  @spec encode_naddr(integer(), String.t(), String.t(), [String.t()]) ::
          {:ok, String.t()} | {:error, term()}
  def encode_naddr(kind, pubkey_hex, identifier, relays \\ []) do
    case nip19_encode_naddr_nif(kind, String.downcase(pubkey_hex), to_string(identifier), relays) do
      {:error, reason} -> {:error, reason}
      encoded -> {:ok, encoded}
    end
  end

  @spec encode_nevent(String.t(), String.t() | nil, [String.t()]) ::
          {:ok, String.t()} | {:error, term()}
  def encode_nevent(event_id_hex, author_hex \\ nil, relays \\ []) do
    author = if author_hex, do: String.downcase(author_hex), else: nil

    case nip19_encode_nevent_nif(String.downcase(event_id_hex), author, relays) do
      {:error, reason} -> {:error, reason}
      encoded -> {:ok, encoded}
    end
  end

  @spec encode_nprofile(String.t(), [String.t()]) :: {:ok, String.t()} | {:error, term()}
  def encode_nprofile(pubkey_hex, relays \\ []) do
    case nip19_encode_nprofile_nif(String.downcase(pubkey_hex), relays) do
      {:error, reason} -> {:error, reason}
      encoded -> {:ok, encoded}
    end
  end

  @spec decode(String.t()) :: {:ok, atom(), map()} | {:error, term()}
  def decode(encoded) when is_binary(encoded) do
    case nip19_decode_address_nif(encoded) do
      {:error, reason} ->
        {:error, reason}

      json ->
        case Jason.decode(json) do
          {:ok, %{"type" => "naddr"} = data} ->
            {:ok, :naddr,
             %{
               kind: data["kind"],
               pubkey: data["pubkey"],
               identifier: data["identifier"],
               relays: data["relays"] || []
             }}

          {:ok, %{"type" => "nevent"} = data} ->
            {:ok, :nevent,
             %{
               event_id: data["event_id"],
               author: data["author"],
               kind: data["kind"],
               relays: data["relays"] || []
             }}

          {:ok, %{"type" => "nprofile"} = data} ->
            {:ok, :nprofile,
             %{
               pubkey: data["pubkey"],
               relays: data["relays"] || []
             }}

          {:ok, _} ->
            {:error, :unsupported_hrp}

          {:error, reason} ->
            {:error, reason}
        end
    end
  end
end

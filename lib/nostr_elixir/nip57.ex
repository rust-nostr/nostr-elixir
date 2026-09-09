defmodule NostrElixir.Nip57 do
  @moduledoc """
  NIP-57: Lightning Zaps (public zap requests).

  Private and anonymous zaps were removed upstream in `nostr` 0.45
  (see rust-nostr changelog / PR #1355).
  """

  defmodule ZapRequestData do
    @moduledoc """
    Struct for NIP-57 Zap Request Data.
    """
    defstruct [
      :public_key,
      :relays,
      :message,
      :amount,
      :lnurl,
      :event_id,
      :event_coordinate
    ]

    @type t :: %__MODULE__{
            public_key: String.t(),
            relays: [String.t()],
            message: String.t(),
            amount: integer() | nil,
            lnurl: String.t() | nil,
            event_id: String.t() | nil,
            event_coordinate: String.t() | nil
          }

    @doc """
    Build a new ZapRequestData struct.
    """
    def new(opts) when is_list(opts) do
      struct(__MODULE__, opts)
    end
  end

  @doc """
  Create a public zap request event (returns JSON string).
  """
  def zap_request(%ZapRequestData{} = data, secret_key_hex) do
    case NostrElixir.nip57_zap_request_nif(
           data.public_key,
           data.relays,
           data.message || "",
           data.amount,
           data.lnurl,
           data.event_id,
           data.event_coordinate,
           secret_key_hex
         ) do
      {:error, reason} -> raise ArgumentError, "NIP-57 zap_request failed: #{reason}"
      result -> result
    end
  end
end

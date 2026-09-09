defmodule NostrElixir.Nip19.AddressTest do
  use ExUnit.Case, async: true
  alias NostrElixir.Nip19.Address

  test "round-trips naddr and ignores extra relay identity" do
    pubkey = String.duplicate("aa", 32)
    {:ok, encoded} = Address.encode_naddr(30023, pubkey, "hello", ["wss://relay.example"])
    assert String.starts_with?(encoded, "naddr1")
    assert {:ok, :naddr, data} = Address.decode(encoded)
    assert data.kind == 30023
    assert data.pubkey == pubkey
    assert data.identifier == "hello"
    assert data.relays == ["wss://relay.example"]

    {:ok, other} = Address.encode_naddr(30023, pubkey, "hello", ["wss://other.example"])
    {:ok, :naddr, a} = Address.decode(encoded)
    {:ok, :naddr, b} = Address.decode(other)
    assert {a.kind, a.pubkey, a.identifier} == {b.kind, b.pubkey, b.identifier}
  end

  test "round-trips nevent and nprofile" do
    pubkey = String.duplicate("bb", 32)
    event_id = String.duplicate("cc", 32)

    {:ok, nevent} = Address.encode_nevent(event_id, pubkey, ["wss://relay.example"])
    assert String.starts_with?(nevent, "nevent1")
    assert {:ok, :nevent, event_data} = Address.decode(nevent)
    assert event_data.event_id == event_id
    assert event_data.author == pubkey

    {:ok, nprofile} = Address.encode_nprofile(pubkey, ["wss://relay.example"])
    assert String.starts_with?(nprofile, "nprofile1")
    assert {:ok, :nprofile, profile_data} = Address.decode(nprofile)
    assert profile_data.pubkey == pubkey
  end
end

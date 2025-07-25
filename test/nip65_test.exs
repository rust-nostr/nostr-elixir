defmodule NostrElixir.Nip65Test do
  use ExUnit.Case, async: true
  alias NostrElixir.Nip65

  @pubkey "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"

  test "create and extract relay list event round-trip" do
    relays = [
      {"wss://relay1.example.com", "read"},
      {"wss://relay2.example.com", "write"},
      {"wss://relay3.example.com", nil}
    ]
    event_json = Nip65.create_relay_list_event(relays, @pubkey)
    assert is_binary(event_json)
    extracted = Nip65.extract_relay_list(event_json)
    assert Enum.sort(extracted) == Enum.sort(relays)
  end

  test "create relay list event with empty list" do
    event_json = Nip65.create_relay_list_event([], @pubkey)
    assert is_binary(event_json)
    extracted = Nip65.extract_relay_list(event_json)
    assert extracted == []
  end

  test "extract relay list from event with only nil metadata" do
    relays = [
      {"wss://relay4.example.com", nil},
      {"wss://relay5.example.com", nil}
    ]
    event_json = Nip65.create_relay_list_event(relays, @pubkey)
    extracted = Nip65.extract_relay_list(event_json)
    assert Enum.sort(extracted) == Enum.sort(relays)
  end

  test "Relay struct conversion and pretty_print" do
    alias NostrElixir.Nip65.Relay
    tuple = {"wss://relay.example.com", "read"}
    struct = Nip65.tuple_to_struct(tuple)
    assert %Relay{url: "wss://relay.example.com", metadata: "read"} = struct
    assert Nip65.struct_to_tuple(struct) == tuple

    # pretty_print for tuple
    assert Nip65.pretty_print([tuple]) == "- wss://relay.example.com (read)"
    # pretty_print for struct
    assert Nip65.pretty_print([struct]) == "- wss://relay.example.com (read)"
    # pretty_print for nil metadata
    assert Nip65.pretty_print([{"wss://relay2.example.com", nil}]) == "- wss://relay2.example.com"
    assert Nip65.pretty_print([%Relay{url: "wss://relay2.example.com", metadata: nil}]) == "- wss://relay2.example.com"
  end

  test "relay url and metadata validation" do
    assert Nip65.valid_relay_url?("wss://relay.example.com")
    refute Nip65.valid_relay_url?("http://relay.example.com")
    refute Nip65.valid_relay_url?("wss://r")
    assert Nip65.valid_metadata?("read")
    assert Nip65.valid_metadata?("write")
    assert Nip65.valid_metadata?(nil)
    refute Nip65.valid_metadata?("other")
    refute Nip65.valid_metadata?(123)
  end
end

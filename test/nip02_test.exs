defmodule NostrElixir.Nip02Test do
  use ExUnit.Case, async: true
  alias NostrElixir.Event
  alias NostrElixir.Keys
  alias NostrElixir.Nip02
  alias NostrElixir.Nip02.Follow

  @alice "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  @bob "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
  @carol "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
  @dave "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
  @eve "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
  @foo "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
  @bar "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"

  setup do
    {:ok, keys: Keys.generate_keypair()}
  end

  test "create and extract follow list event round-trip", %{keys: keys} do
    follows = [
      {@alice, "wss://relay1.example.com", "Alice"},
      {@bob, nil, nil},
      {@carol, "wss://relay2.example.com", nil}
    ]

    event_json = Nip02.create_follow_list_event(follows, keys)
    assert is_binary(event_json)
    event = Jason.decode!(event_json)
    assert event["pubkey"] == keys.public_key
    assert Event.verify(event_json) == true
    extracted = Nip02.extract_follows(event_json)
    assert Enum.sort(extracted) == Enum.sort(follows)
  end

  test "create follow list event with empty list", %{keys: keys} do
    event_json = Nip02.create_follow_list_event([], keys.secret_key)
    assert is_binary(event_json)
    assert Event.verify(event_json) == true
    extracted = Nip02.extract_follows(event_json)
    assert extracted == []
  end

  test "extract follow list with only nil fields", %{keys: keys} do
    follows = [
      {@dave, nil, nil},
      {@eve, nil, nil}
    ]

    event_json = Nip02.create_follow_list_event(follows, keys.secret_key)
    extracted = Nip02.extract_follows(event_json)
    assert Enum.sort(extracted) == Enum.sort(follows)
  end

  test "rejects an invalid secret key" do
    assert_raise ArgumentError, ~r/NIP-02 create_follow_list_event failed/, fn ->
      Nip02.create_follow_list_event([], "not-a-secret-key")
    end
  end

  test "tuple <-> struct conversion" do
    tuple = {@foo, "wss://relay.example.com", "Foo"}
    struct = Nip02.tuple_to_struct(tuple)
    assert struct == %Follow{pubkey: @foo, relay_url: "wss://relay.example.com", alias: "Foo"}
    assert Nip02.struct_to_tuple(struct) == tuple
  end

  test "pretty print follow list" do
    follows = [
      %Follow{pubkey: @foo, relay_url: "wss://relay.example.com", alias: "Foo"},
      {@bar, nil, "Bar"}
    ]

    output = Nip02.pretty_print(follows)
    assert output =~ @foo
    assert output =~ "wss://relay.example.com"
    assert output =~ "(Foo)"
    assert output =~ @bar
    assert output =~ "(Bar)"
  end
end

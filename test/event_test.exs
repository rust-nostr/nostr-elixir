defmodule NostrElixir.EventTest do
  use ExUnit.Case, async: true
  alias NostrElixir.Event
  alias NostrElixir.Keys

  test "new_event creates an unsigned event" do
    keys = Keys.generate_keypair()
    event_json = Event.new(keys.public_key, "Hello, Nostr!", 1, [])
    event = Jason.decode!(event_json)

    assert event["pubkey"] == keys.public_key
    assert event["content"] == "Hello, Nostr!"
    assert event["kind"] == 1
    assert event["sig"] == ""
    assert is_binary(event["id"])
  end

  test "new_event with tags" do
    keys = Keys.generate_keypair()
    tags = [["t", "hello"], ["p", keys.public_key]]
    event_json = Event.new(keys.public_key, "Hello", 1, tags)
    event = Jason.decode!(event_json)

    assert is_list(event["tags"])
    assert length(event["tags"]) > 0
  end

  test "verify_event rejects unsigned events" do
    keys = Keys.generate_keypair()
    event_json = Event.new(keys.public_key, "Hello", 1, [])

    assert_raise ArgumentError, ~r/Failed to verify event/, fn ->
      Event.verify(event_json)
    end
  end

  test "sign_event signs and verify_event verifies the event" do
    keys = Keys.generate_keypair()
    event_json = Event.new(keys.public_key, "Hello, Nostr!", 1, [])
    signed_event_json = Event.sign(event_json, keys.secret_key)
    assert Event.verify(signed_event_json) == true
  end

  test "event_to_json and event_from_json work" do
    keys = Keys.generate_keypair()
    event_json = Event.new(keys.public_key, "Hello", 1, [])
    assert Event.to_json(event_json) == event_json
    assert Event.from_json(event_json) == event_json
  end

  test "create_text_note creates and signs a text note" do
    keys = Keys.generate_keypair()
    signed_event_json = Event.create_text_note(keys, "Hello, Nostr!")
    assert Event.verify(signed_event_json) == true
  end

  test "create_metadata creates and signs a metadata event" do
    keys = Keys.generate_keypair()
    metadata = %{name: "Alice", about: "Nostr user"}
    signed_event_json = Event.create_metadata(keys, metadata)
    assert Event.verify(signed_event_json) == true
  end
end

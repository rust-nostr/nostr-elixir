defmodule NostrElixir.Nip57Test do
  use ExUnit.Case, async: true
  alias NostrElixir.Nip57
  alias NostrElixir.Nip57.ZapRequestData

  @alice_secret_key "5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a"

  describe "ZapRequestData.new/1" do
    test "builds struct with all fields" do
      data =
        ZapRequestData.new(
          public_key: "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
          relays: ["wss://relay.damus.io"],
          message: "Zap!",
          amount: 1234,
          lnurl: "lnurl1...",
          event_id: "eventid",
          event_coordinate: "coord"
        )

      assert data.public_key == "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4"
      assert data.relays == ["wss://relay.damus.io"]
      assert data.message == "Zap!"
      assert data.amount == 1234
      assert data.lnurl == "lnurl1..."
      assert data.event_id == "eventid"
      assert data.event_coordinate == "coord"
    end
  end

  describe "zap_request/2" do
    test "returns a valid event for placeholder keys" do
      data =
        ZapRequestData.new(
          public_key: "b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4",
          relays: ["wss://relay.damus.io"],
          message: "Thanks!",
          amount: 1000
        )

      result = Nip57.zap_request(data, @alice_secret_key)
      assert is_binary(result)
      assert String.length(result) > 0
      event = Jason.decode!(result)
      assert event["kind"] == 9734
    end
  end
end

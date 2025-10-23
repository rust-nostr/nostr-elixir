defmodule NostrElixir.Nip04Test do
  use ExUnit.Case, async: true
  alias NostrElixir.Nip04
  alias NostrElixir.Keys

  test "encrypt and decrypt round-trip" do
    keys1 = Keys.generate_keypair()
    keys2 = Keys.generate_keypair()
    plaintext = "hello nip04!"
    ciphertext = Nip04.encrypt(keys1.secret_key, keys2.public_key, plaintext)
    assert is_binary(ciphertext)
    decrypted = Nip04.decrypt(keys2.secret_key, keys1.public_key, ciphertext)
    assert decrypted == plaintext
  end

  test "decrypt with wrong key fails" do
    keys1 = Keys.generate_keypair()
    keys2 = Keys.generate_keypair()
    keys3 = Keys.generate_keypair()
    plaintext = "test"
    ciphertext = Nip04.encrypt(keys1.secret_key, keys2.public_key, plaintext)
    assert_raise ArgumentError, ~r/NIP-04 decrypt failed/, fn ->
      Nip04.decrypt(keys3.secret_key, keys1.public_key, ciphertext)
    end
  end
end

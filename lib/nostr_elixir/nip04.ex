defmodule NostrElixir.Nip04 do
  @moduledoc """
  NIP-04: Encrypted Direct Messages (AES-256-CBC)

  This module provides functions to encrypt and decrypt messages using NIP-04.

  ## Examples

      iex> alias NostrElixir.Nip04
      iex> sk = "...hex secret key..."
      iex> pk = "...hex public key..."
      iex> ciphertext = Nip04.encrypt(sk, pk, "hello!")
      iex> is_binary(ciphertext)
      true
      iex> Nip04.decrypt(sk, pk, ciphertext)
      "hello!"

  """

  @doc """
  Encrypt a message using NIP-04.

  ## Parameters
    * `secret_key` - hex string of sender's secret key
    * `public_key` - hex string of recipient's public key
    * `content` - plaintext message

  ## Returns
    * base64-encoded payload string ("{ciphertext}?iv=..." format)
  """
  def encrypt(secret_key, public_key, content) do
    case NostrElixir.nip04_encrypt_nif(secret_key, public_key, content) do
      {:error, reason} -> raise ArgumentError, "NIP-04 encrypt failed: #{reason}"
      result -> result
    end
  end

  @doc """
  Decrypt a NIP-04 message.

  ## Parameters
    * `secret_key` - hex string of recipient's secret key
    * `public_key` - hex string of sender's public key
    * `payload` - base64-encoded payload string ("{ciphertext}?iv=..." format)

  ## Returns
    * plaintext message (string)
  """
  def decrypt(secret_key, public_key, payload) do
    case NostrElixir.nip04_decrypt_nif(secret_key, public_key, payload) do
      {:error, reason} -> raise ArgumentError, "NIP-04 decrypt failed: #{reason}"
      result -> result
    end
  end
end

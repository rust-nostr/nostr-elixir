defmodule NostrElixir.Nip23 do
  @moduledoc """
  NIP-23: Long-form Content

  This module provides helpers for creating long-form content events (kind 30023) according to NIP-23.

  **Implementation note:**
  > Tag construction is implemented in pure Elixir for full NIP-23 spec compliance. This ensures correct and predictable behavior for all required and optional tags, and makes it easy to adapt to future spec changes.

  See: https://github.com/nostr-protocol/nips/blob/master/23.md

  ## Examples

      iex> keys = NostrElixir.Keys.generate_keypair()
      iex> opts = %{
      ...>   title: "My Article",
      ...>   summary: "A brief summary",
      ...>   image: "https://example.com/image.jpg",
      ...>   published_at: 1234567890
      ...> }
      iex> content = "# My Article\\n\\nThis is the full article content in Markdown..."
      iex> event_json = NostrElixir.Nip23.create_long_form(keys, content, opts)
      iex> NostrElixir.Event.verify(event_json)
      true
  """

  defmodule LongForm do
    @moduledoc """
    Struct representing a NIP-23 long-form content event.
    """
    defstruct [
      :title,
      :summary,
      :image,
      :published_at,
      :hashtags,
      :canonical,
      :lang,
      :content
    ]
  end

  @doc """
  Create and sign a long-form content event (kind 30023).

  ## Options

  - `:title` - (required) The article title
  - `:summary` - (optional) A brief summary of the article
  - `:image` - (optional) URL to a cover image
  - `:published_at` - (optional) Unix timestamp when the article was published
  - `:hashtags` - (optional) List of hashtags (without the # symbol)
  - `:canonical` - (optional) Canonical URL for the article
  - `:lang` - (optional) Language code (e.g., "en", "es")

  ## Examples
      iex> keys = NostrElixir.Keys.generate_keypair()
      iex> opts = %{title: "My Article"}
      iex> content = "# My Article\\n\\nContent here..."
      iex> event_json = NostrElixir.Nip23.create_long_form(keys, content, opts)
      iex> is_binary(event_json)
      true
  """
  def create_long_form(keys, content, opts \\ %{}) do
    tags = build_long_form_tags(opts)
    create_and_sign_event(keys, content, 30023, tags)
  end

  @doc """
  Extract metadata from a long-form content event JSON.
  Returns a %LongForm{} struct with the extracted data.

  ## Examples
      iex> event_json = "{\"kind\": 30023, \"content\": \"# Title\\nContent\", \"tags\": [[\"title\", \"Title\"]]}"
      iex> metadata = NostrElixir.Nip23.extract_metadata(event_json)
      iex> metadata.title
      "Title"
  """
  def extract_metadata(event_json) when is_binary(event_json) do
    event = Jason.decode!(event_json)
    tags = event["tags"] || []

    %LongForm{
      title: find_tag_value(tags, "title"),
      summary: find_tag_value(tags, "summary"),
      image: find_tag_value(tags, "image"),
      published_at: find_tag_value(tags, "published_at") |> parse_timestamp(),
      hashtags: find_tag_values(tags, "t"),
      canonical: find_tag_value(tags, "canonical"),
      lang: find_tag_value(tags, "lang"),
      content: event["content"]
    }
  end

  @doc """
  Pretty-print a long-form content event (shows title, summary, and content preview).
  """
  def pretty_print(event_json) when is_binary(event_json) do
    metadata = extract_metadata(event_json)
    event = Jason.decode!(event_json)

    """
    📄 Long-form Content Event
    ─────────────────────────
    Title: #{metadata.title || "Untitled"}
    #{if metadata.summary, do: "Summary: #{metadata.summary}", else: ""}
    #{if metadata.published_at, do: "Published: #{metadata.published_at}", else: ""}
    #{if length(metadata.hashtags) > 0, do: "Tags: #{Enum.join(metadata.hashtags, ", ")}", else: ""}
    #{if metadata.lang, do: "Language: #{metadata.lang}", else: ""}

    Content Preview:
    #{String.slice(metadata.content || "", 0, 200)}#{if String.length(metadata.content || "") > 200, do: "...", else: ""}

    Event ID: #{event["id"]}
    Author: #{event["pubkey"]}
    """
  end

  @doc """
  Build NIP-23 tags from options map.
  """
      def build_long_form_tags(opts) do
    # Add hashtags first
    tags = if opts[:hashtags] do
      Enum.map(opts.hashtags, fn tag -> ["t", tag] end)
    else
      []
    end

    # Add other tags
    tags = if opts[:title], do: tags ++ [["title", opts.title]], else: tags
    tags = if opts[:summary], do: tags ++ [["summary", opts.summary]], else: tags
    tags = if opts[:image], do: tags ++ [["image", opts.image]], else: tags
    tags = if opts[:published_at], do: tags ++ [["published_at", to_string(opts.published_at)]], else: tags
    tags = if opts[:canonical], do: tags ++ [["canonical", opts.canonical]], else: tags
    tags = if opts[:lang], do: tags ++ [["lang", opts.lang]], else: tags

    tags
  end

  defp create_and_sign_event(keys, content, kind, tags) do
    tags_json = Jason.encode!(tags)
    pubkey = if is_map(keys), do: keys.public_key, else: Jason.decode!(keys)["public_key"]
    event_json = NostrElixir.event_new_nif(pubkey, content, kind, tags_json)
    secret_key = if is_map(keys), do: keys.secret_key, else: Jason.decode!(keys)["secret_key"]
    NostrElixir.event_sign_nif(event_json, secret_key)
  end

  defp find_tag_value(tags, tag_name) do
    case Enum.find(tags, fn tag -> List.first(tag) == tag_name end) do
      nil -> nil
      tag -> Enum.at(tag, 1)
    end
  end

  defp find_tag_values(tags, tag_name) do
    tags
    |> Enum.filter(fn tag -> List.first(tag) == tag_name end)
    |> Enum.map(fn tag -> Enum.at(tag, 1) end)
  end

  defp parse_timestamp(nil), do: nil
  defp parse_timestamp(timestamp_str) when is_binary(timestamp_str) do
    case Integer.parse(timestamp_str) do
      {timestamp, _} -> timestamp
      :error -> nil
    end
  end
  defp parse_timestamp(timestamp) when is_integer(timestamp), do: timestamp
  defp parse_timestamp(_), do: nil
end

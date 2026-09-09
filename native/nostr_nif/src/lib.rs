use rustler::NifResult;
use nostr::prelude::*;
use serde_json;
use std::str::FromStr;
use rustler::Encoder;

// NIP-65: Relay List Metadata NIFs
#[rustler::nif]
fn nip65_create_relay_list_event_nif(
    relays: Vec<(String, Option<String>)>,
    secret_key: String,
) -> NifResult<String> {
    let secret_key = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let keys = Keys::new(secret_key);
    let relays = relays
        .into_iter()
        .map(|(url, meta)| {
            let relay_url = to_rustler_error(RelayUrl::parse(&url))?;
            let metadata = match meta {
                Some(m) => Some(to_rustler_error(RelayMetadata::from_str(&m))?),
                None => None,
            };
            Ok((relay_url, metadata))
        })
        .collect::<NifResult<Vec<_>>>()?;
    let event = to_rustler_error(RelayList::new(relays).finalize(&keys))?;
    Ok(event.as_json())
}

#[rustler::nif]
fn nip65_extract_relay_list_nif<'a>(env: rustler::Env<'a>, event_json_term: rustler::Term<'a>) -> rustler::Term<'a> {
    let event_json: String = rustler::Decoder::decode(event_json_term).unwrap();
    let event = Event::from_json(&event_json).unwrap();
    let relays: Vec<(String, Option<String>)> = nip65::extract_relay_list(&event)
        .map(|(url, meta)| (url.to_string(), meta.map(|m| m.as_str().to_string())))
        .collect();
    relays.encode(env)
}

#[rustler::nif]
fn nip02_create_contact_list_event_nif(
    contacts: Vec<(String, Option<String>, Option<String>)>,
    secret_key: String,
) -> NifResult<String> {
    let secret_key = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let keys = Keys::new(secret_key);
    let contacts = contacts
        .into_iter()
        .map(|(pk, relay_url, alias)| {
            Ok(Contact {
                public_key: to_rustler_error(PublicKey::from_str(&pk))?,
                relay_url: match relay_url {
                    Some(url) => Some(to_rustler_error(RelayUrl::parse(&url))?),
                    None => None,
                },
                alias,
            })
        })
        .collect::<NifResult<Vec<_>>>()?;
    let event = to_rustler_error(ContactListBuilder::new(contacts).finalize(&keys))?;
    Ok(event.as_json())
}

#[rustler::nif]
fn nip02_extract_contacts_nif<'a>(env: rustler::Env<'a>, event_json_term: rustler::Term<'a>) -> rustler::Term<'a> {
    let event_json: String = rustler::Decoder::decode(event_json_term).unwrap();
    let event = Event::from_json(&event_json).unwrap();
    let follows: Vec<(String, Option<String>, Option<String>)> = event.tags.to_vec().into_iter().filter_map(|tag| {
        let tag_vec = tag.to_vec();
        if tag_vec.len() >= 2 && tag_vec[0] == "p" {
            let pk = tag_vec[1].clone();
            let relay_url = tag_vec.get(2).cloned().filter(|s| !s.is_empty());
            let alias = tag_vec.get(3).cloned().filter(|s| !s.is_empty());
            Some((pk, relay_url, alias))
        } else {
            None
        }
    }).collect();
    follows.encode(env)
}

#[rustler::nif]
fn nip10_create_text_note_nif(keys_json: String, content: String) -> NifResult<String> {
    let keys_map: serde_json::Value = serde_json::from_str(&keys_json).unwrap();
    let secret_key = SecretKey::from_str(keys_map["secret_key"].as_str().unwrap()).unwrap();
    let keys = Keys::new(secret_key);
    let event = EventBuilder::new(Kind::TextNote, content)
        .finalize(&keys)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(event.as_json())
}

#[rustler::nif]
fn nip10_create_text_note_reply_nif(
    keys_json: String,
    content: String,
    reply_to_json: String,
    root_json: Option<String>,
    relay_url: Option<String>,
) -> NifResult<String> {
    let keys_map: serde_json::Value = serde_json::from_str(&keys_json).unwrap();
    let secret_key = SecretKey::from_str(keys_map["secret_key"].as_str().unwrap()).unwrap();
    let keys = Keys::new(secret_key);
    let reply_to = Event::from_json(&reply_to_json).unwrap();
    let root = match root_json {
        Some(json) => Some(Event::from_json(&json).unwrap()),
        None => None,
    };
    let mut builder = TextNoteReplyBuilder::new(content, &reply_to);
    if let Some(root) = root.as_ref() {
        builder = builder.root(root);
    }
    if let Some(url) = relay_url.and_then(|url| RelayUrl::parse(&url).ok()) {
        builder = builder.relay_hint(url);
    }
    let event = builder
        .finalize(&keys)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(event.as_json())
}

rustler::init!("Elixir.NostrElixir");

// Helper function to convert nostr errors to rustler errors
fn to_rustler_error<T>(result: Result<T, impl std::fmt::Display>) -> NifResult<T> {
    result.map_err(|e| rustler::Error::Term(Box::new(e.to_string())))
}

#[rustler::nif]
fn keys_generate_nif() -> NifResult<String> {
    let keys = Keys::generate();
    let public_key = keys.public_key();
    let secret_key = keys.secret_key();

    let result = serde_json::json!({
        "public_key": public_key.to_string(),
        "secret_key": secret_key.to_secret_hex()
    });

    Ok(result.to_string())
}

#[rustler::nif]
fn keys_parse_nif(secret_key_str: String) -> NifResult<String> {
    let keys = to_rustler_error(Keys::parse(&secret_key_str))?;
    let public_key = keys.public_key();
    let secret_key = keys.secret_key();

    let result = serde_json::json!({
        "public_key": public_key.to_string(),
        "secret_key": secret_key.to_secret_hex()
    });

    Ok(result.to_string())
}

#[rustler::nif]
fn keys_public_key_nif(keys_json: String) -> NifResult<String> {
    let keys: serde_json::Value = to_rustler_error(serde_json::from_str(&keys_json))?;
    Ok(keys["public_key"].as_str().unwrap_or("").to_string())
}

#[rustler::nif]
fn keys_secret_key_nif(keys_json: String) -> NifResult<String> {
    let keys: serde_json::Value = to_rustler_error(serde_json::from_str(&keys_json))?;
    Ok(keys["secret_key"].as_str().unwrap_or("").to_string())
}

#[rustler::nif]
fn keys_public_key_bech32_nif(public_key_str: String) -> NifResult<String> {
    let public_key = to_rustler_error(PublicKey::from_hex(&public_key_str))?;
    to_rustler_error(public_key.to_bech32())
}

#[rustler::nif]
fn keys_secret_key_bech32_nif(secret_key_str: String) -> NifResult<String> {
    let secret_key = to_rustler_error(SecretKey::from_hex(&secret_key_str))?;
    to_rustler_error(secret_key.to_bech32())
}

#[rustler::nif]
fn keys_secret_key_hex_nif(secret_key_str: String) -> NifResult<String> {
    let secret_key = to_rustler_error(SecretKey::from_bech32(&secret_key_str))?;
    Ok(hex::encode(secret_key.as_secret_bytes()))
}

#[rustler::nif]
fn parser_parse_nif(text: String) -> NifResult<String> {
    let parser = NostrParser::new();
    let tokens = parser.parse(&text);

    let mut result = Vec::new();
    for token in tokens {
        match token {
            Token::Text(text) => {
                result.push(serde_json::json!({
                    "token_type": "text",
                    "value": text.to_string()
                }));
            }
            Token::Url(url) => {
                result.push(serde_json::json!({
                    "token_type": "url",
                    "value": url.to_string()
                }));
            }
            Token::Hashtag(tag) => {
                result.push(serde_json::json!({
                    "token_type": "hashtag",
                    "value": tag.to_string()
                }));
            }
            _ => {
                result.push(serde_json::json!({
                    "token_type": "text",
                    "value": format!("{:?}", token)
                }));
            }
        }
    }

    Ok(serde_json::to_string(&result).unwrap())
}

#[rustler::nif]
fn nip19_encode_nif(data_type: String, data: String) -> NifResult<String> {
    match data_type.as_str() {
        "npub" => {
            let pubkey = to_rustler_error(PublicKey::from_hex(&data))?;
            to_rustler_error(pubkey.to_bech32())
        }
        "nsec" => {
            let secret_key = to_rustler_error(SecretKey::from_hex(&data))?;
            to_rustler_error(secret_key.to_bech32())
        }
        "note" => {
            let event_id = to_rustler_error(EventId::from_hex(&data))?;
            to_rustler_error(event_id.to_bech32())
        }
        _ => Err(rustler::Error::BadArg),
    }
}

#[rustler::nif]
fn nip19_decode_nif(bech32_string: String) -> NifResult<String> {
    let result = if bech32_string.starts_with("npub") {
        let pubkey = to_rustler_error(PublicKey::from_bech32(&bech32_string))?;
        serde_json::json!({
            "data_type": "npub",
            "data": pubkey.to_string()
        })
    } else if bech32_string.starts_with("nsec") {
        let secret_key = to_rustler_error(SecretKey::from_bech32(&bech32_string))?;
        serde_json::json!({
            "data_type": "nsec",
            "data": hex::encode(secret_key.as_secret_bytes())
        })
    } else if bech32_string.starts_with("note") {
        let event_id = to_rustler_error(EventId::from_bech32(&bech32_string))?;
        serde_json::json!({
            "data_type": "note",
            "data": event_id.to_string()
        })
    } else {
        return Err(rustler::Error::Term(Box::new("Unknown bech32 prefix".to_string())));
    };

    Ok(result.to_string())
}

fn parse_relay_urls(relays: Vec<String>) -> NifResult<Vec<RelayUrl>> {
    relays
        .into_iter()
        .map(|relay| RelayUrl::parse(&relay).map_err(|e| rustler::Error::Term(Box::new(e.to_string()))))
        .collect()
}

#[rustler::nif]
fn nip19_encode_naddr_nif(
    kind: u16,
    pubkey_hex: String,
    identifier: String,
    relays: Vec<String>,
) -> NifResult<String> {
    let public_key = to_rustler_error(PublicKey::from_hex(&pubkey_hex))?;
    let relays = parse_relay_urls(relays)?;
    let coordinate = Coordinate::new(Kind::from(kind), public_key).identifier(identifier);
    let naddr = Nip19Coordinate::new(coordinate, relays);
    to_rustler_error(naddr.to_bech32())
}

#[rustler::nif]
fn nip19_encode_nevent_nif(
    event_id_hex: String,
    author_hex: Option<String>,
    relays: Vec<String>,
) -> NifResult<String> {
    let event_id = to_rustler_error(EventId::from_hex(&event_id_hex))?;
    let relays = parse_relay_urls(relays)?;
    let mut nevent = Nip19Event::new(event_id).relays(relays);
    if let Some(author) = author_hex {
        nevent = nevent.author(to_rustler_error(PublicKey::from_hex(&author))?);
    }
    to_rustler_error(nevent.to_bech32())
}

#[rustler::nif]
fn nip19_encode_nprofile_nif(pubkey_hex: String, relays: Vec<String>) -> NifResult<String> {
    let public_key = to_rustler_error(PublicKey::from_hex(&pubkey_hex))?;
    let relays = parse_relay_urls(relays)?;
    let nprofile = Nip19Profile::new(public_key, relays);
    to_rustler_error(nprofile.to_bech32())
}

#[rustler::nif]
fn nip19_decode_address_nif(encoded: String) -> NifResult<String> {
    let decoded = to_rustler_error(Nip19::from_bech32(&encoded))?;
    let result = match decoded {
        Nip19::Coordinate(naddr) => serde_json::json!({
            "type": "naddr",
            "kind": naddr.coordinate.kind.as_u16(),
            "pubkey": naddr.coordinate.public_key.to_string(),
            "identifier": naddr.coordinate.identifier,
            "relays": naddr.relays.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
        }),
        Nip19::Event(nevent) => serde_json::json!({
            "type": "nevent",
            "event_id": nevent.event_id.to_string(),
            "author": nevent.author.map(|pk| pk.to_string()),
            "kind": nevent.kind.map(|k| k.as_u16()),
            "relays": nevent.relays.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
        }),
        Nip19::Profile(nprofile) => serde_json::json!({
            "type": "nprofile",
            "pubkey": nprofile.public_key.to_string(),
            "relays": nprofile.relays.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
        }),
        Nip19::Pubkey(_) | Nip19::Secret(_) | Nip19::EventId(_) => {
            return Err(rustler::Error::Term(Box::new(
                "unsupported_hrp: expected naddr, nevent, or nprofile".to_string(),
            )));
        }
    };
    Ok(result.to_string())
}

#[rustler::nif]
fn event_new_nif(pubkey: String, content: String, kind: u16, tags_json: String) -> NifResult<String> {
    let public_key = to_rustler_error(PublicKey::from_hex(&pubkey))?;
    let kind = Kind::from(kind);

    let tags: Vec<Vec<String>> = to_rustler_error(serde_json::from_str(&tags_json))?;
    let tags: Vec<Tag> = tags.into_iter()
        .map(|tag_vec| {
            to_rustler_error(Tag::parse(tag_vec)).unwrap_or_else(|_| Tag::parse(vec!["t", "unknown"]).unwrap())
        })
        .collect();

    let event = EventBuilder::new(kind, content)
        .tags(tags)
        .finalize_unsigned(public_key);
    let event_id = event.compute_id();

    let result = serde_json::json!({
        "id": event_id.to_string(),
        "pubkey": event.pubkey.to_string(),
        "created_at": event.created_at.as_secs(),
        "kind": event.kind.as_u16(),
        "tags": event.tags.to_vec().into_iter().map(|tag| tag.to_vec()).collect::<Vec<Vec<String>>>(),
        "content": event.content,
        "sig": ""
    });

    Ok(result.to_string())
}

#[rustler::nif]
fn event_sign_nif(event_json: String, secret_key: String) -> NifResult<String> {
    let event_data: serde_json::Value = to_rustler_error(serde_json::from_str(&event_json))?;
    let keys = to_rustler_error(Keys::parse(&secret_key))?;

    let kind = Kind::from(event_data["kind"].as_u64().unwrap_or(0) as u16);
    let content = event_data["content"].as_str().unwrap_or("").to_string();

    let tags: Vec<Vec<String>> = event_data["tags"].as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|tag| tag.as_array().unwrap_or(&Vec::new()).iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
        .collect();

    let tags: Vec<Tag> = tags.into_iter()
        .map(|tag_vec| {
            to_rustler_error(Tag::parse(tag_vec)).unwrap_or_else(|_| Tag::parse(vec!["t", "unknown"]).unwrap())
        })
        .collect();

    let event = EventBuilder::new(kind, content)
        .tags(tags)
        .finalize(&keys)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;

    Ok(event.as_json())
}

#[rustler::nif]
fn event_verify_nif(event_json: String) -> NifResult<bool> {
    let event = to_rustler_error(Event::from_json(&event_json))?;
    match event.verify() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}

#[rustler::nif]
fn event_to_json_nif(event_json: String) -> NifResult<String> {
    Ok(event_json)
}

#[rustler::nif]
fn event_from_json_nif(json_string: String) -> NifResult<String> {
    let event_data: serde_json::Value = to_rustler_error(serde_json::from_str(&json_string))?;

    if !event_data["id"].is_string() || !event_data["pubkey"].is_string() ||
       !event_data["created_at"].is_number() || !event_data["kind"].is_number() ||
       !event_data["content"].is_string() || !event_data["sig"].is_string() {
        return Err(rustler::Error::Term(Box::new("Invalid event JSON: missing required fields".to_string())));
    }

    Ok(json_string)
}

#[rustler::nif]
fn filter_new_nif(filter_spec: String) -> NifResult<String> {
    let filter_data: serde_json::Value = to_rustler_error(serde_json::from_str(&filter_spec))?;
    let mut filter = Filter::new();

    if let Some(authors) = filter_data["authors"].as_array() {
        let pubkeys: Result<Vec<PublicKey>, _> = authors.iter()
            .map(|a| PublicKey::from_hex(a.as_str().unwrap_or("")))
            .collect();
        if let Ok(pubkeys) = pubkeys {
            filter = filter.authors(pubkeys);
        }
    }

    if let Some(kinds) = filter_data["kinds"].as_array() {
        let kinds: Vec<Kind> = kinds.iter()
            .map(|k| Kind::from(k.as_u64().unwrap_or(0) as u16))
            .collect();
        filter = filter.kinds(kinds);
    }

    if let Some(limit) = filter_data["limit"].as_u64() {
        filter = filter.limit(limit as usize);
    }

    if let Some(since) = filter_data["since"].as_u64() {
        filter = filter.since(Timestamp::from(since));
    }

    if let Some(until) = filter_data["until"].as_u64() {
        filter = filter.until(Timestamp::from(until));
    }

    if let Some(search) = filter_data["search"].as_str() {
        filter = filter.search(search);
    }

    if let Some(hashtags) = filter_data["hashtags"].as_array() {
        for hashtag in hashtags {
            if let Some(tag) = hashtag.as_str() {
                filter = filter.hashtag(tag);
            }
        }
    }

    let result = serde_json::json!({
        "authors": filter.authors.map(|a| a.iter().map(|pk| pk.to_string()).collect::<Vec<String>>()),
        "kinds": filter.kinds.map(|k| k.iter().map(|kind| kind.as_u16()).collect::<Vec<u16>>()),
        "limit": filter.limit,
        "since": filter.since.map(|t| t.as_secs()),
        "until": filter.until.map(|t| t.as_secs()),
        "search": filter.search,
        "hashtags": filter_data["hashtags"]
    });

    Ok(result.to_string())
}

#[rustler::nif]
fn filter_to_json_nif(filter_json: String) -> NifResult<String> {
    Ok(filter_json)
}

#[rustler::nif]
fn filter_from_json_nif(json_string: String) -> NifResult<String> {
    let _filter_data: serde_json::Value = to_rustler_error(serde_json::from_str(&json_string))?;
    Ok(json_string)
}

// NIP-06: Mnemonic/HD Wallet Support

#[rustler::nif]
fn nip06_generate_mnemonic_nif(word_count: u32) -> NifResult<String> {
    use bip39::{Mnemonic, Language};
    use rand::RngCore;
    let entropy_bytes = match word_count {
        12 => 16,
        15 => 20,
        18 => 24,
        21 => 28,
        24 => 32,
        _ => return Err(rustler::Error::Term(Box::new("Invalid word count. Must be 12, 15, 18, 21, or 24".to_string())))
    };
    let mut entropy = vec![0u8; entropy_bytes];
    rand::thread_rng().fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let phrase = mnemonic.to_string();
    let result = serde_json::json!({
        "phrase": phrase,
        "word_count": word_count
    });
    Ok(result.to_string())
}

#[rustler::nif]
fn nip06_mnemonic_to_seed_nif(mnemonic_phrase: String, passphrase: Option<String>) -> NifResult<String> {
    use bip39::{Mnemonic, Language};
    let mnemonic = to_rustler_error(Mnemonic::parse_in_normalized(Language::English, &mnemonic_phrase))?;
    let passphrase = passphrase.unwrap_or_default();
    let seed = mnemonic.to_seed_normalized(&passphrase);
    let result = serde_json::json!({
        "seed": hex::encode(seed),
        "seed_length": seed.len()
    });
    Ok(result.to_string())
}

#[rustler::nif]
fn nip06_derive_key_nif(seed_hex: String, derivation_path: String) -> NifResult<String> {
    use bitcoin::bip32::{DerivationPath, ExtendedPrivKey};
    use bitcoin::Network;
    use bitcoin::secp256k1::Secp256k1;
    let seed = to_rustler_error(hex::decode(&seed_hex))?;
    let secp = Secp256k1::new();
    let master_key = to_rustler_error(ExtendedPrivKey::new_master(Network::Bitcoin, &seed))?;
    let derivation_path = to_rustler_error(DerivationPath::from_str(&derivation_path))?;
    let derived_key = to_rustler_error(master_key.derive_priv(&secp, &derivation_path))?;
    let secret_key = derived_key.private_key;
    let public_key = secret_key.public_key(&secp);
    let result = serde_json::json!({
        "public_key": public_key.to_string(),
        "secret_key": hex::encode(secret_key.secret_bytes()),
        "derivation_path": derivation_path.to_string()
    });
    Ok(result.to_string())
}

#[rustler::nif]
fn nip06_validate_mnemonic_nif(mnemonic_phrase: String) -> NifResult<bool> {
    use bip39::{Mnemonic, Language};
    match Mnemonic::parse_in_normalized(Language::English, &mnemonic_phrase) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}

#[rustler::nif]
fn nip04_encrypt_nif(secret_key: String, public_key: String, plaintext: String) -> NifResult<String> {
    use nostr::nips::nip04;
    let sk = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let pk = to_rustler_error(PublicKey::from_str(&public_key))?;
    let ciphertext = nip04::encrypt(&sk, &pk, plaintext)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(ciphertext)
}

#[rustler::nif]
fn nip04_decrypt_nif(secret_key: String, public_key: String, payload: String) -> NifResult<String> {
    use nostr::nips::nip04;
    let sk = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let pk = to_rustler_error(PublicKey::from_str(&public_key))?;
    let plaintext = nip04::decrypt(&sk, &pk, payload)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(plaintext)
}

#[rustler::nif]
fn nip44_encrypt_nif(secret_key: String, public_key: String, content: String) -> NifResult<String> {
    use nostr::nips::nip44::{self, Version};
    let sk = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let pk = to_rustler_error(PublicKey::from_str(&public_key))?;
    let ciphertext = nip44::encrypt(&sk, &pk, content, Version::V2)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(ciphertext)
}

#[rustler::nif]
fn nip44_decrypt_nif(secret_key: String, public_key: String, payload: String) -> NifResult<String> {
    use nostr::nips::nip44;
    let sk = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let pk = to_rustler_error(PublicKey::from_str(&public_key))?;
    let plaintext = nip44::decrypt(&sk, &pk, payload)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(plaintext)
}

/// Public zap request (private/anonymous zaps were removed upstream in nostr 0.45).
#[rustler::nif]
fn nip57_zap_request_nif(
    public_key: String,
    relays: Vec<String>,
    message: String,
    amount: Option<u64>,
    lnurl: Option<String>,
    event_id: Option<String>,
    event_coordinate: Option<String>,
    secret_key_hex: String
) -> NifResult<String> {
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let relays: Result<Vec<RelayUrl>, _> = relays.iter().map(|r| RelayUrl::parse(r)).collect();
    let relays = relays.map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let mut data = ZapRequestData::new(pk, relays).message(message);
    if let Some(a) = amount { data = data.amount(a); }
    if let Some(l) = lnurl { data = data.lnurl(l); }
    if let Some(eid) = event_id {
        data = data.event_id(EventId::from_hex(&eid).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?);
    }
    if let Some(coord) = event_coordinate {
        data = data.event_coordinate(Coordinate::from_str(&coord).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?);
    }
    let sk = SecretKey::from_str(&secret_key_hex).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let keys = Keys::new(sk);
    let event = data
        .finalize(&keys)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(event.as_json())
}

#[rustler::nif]
fn nip17_encrypt_dm_nif(secret_key: String, public_key: String, plaintext: String) -> NifResult<String> {
    use nostr::nips::nip04;
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let ciphertext = nip04::encrypt(&sk, &pk, plaintext)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(ciphertext)
}

#[rustler::nif]
fn nip17_decrypt_dm_nif(secret_key: String, public_key: String, ciphertext: String) -> NifResult<String> {
    use nostr::nips::nip04;
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let plaintext = nip04::decrypt(&sk, &pk, ciphertext)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(plaintext)
}

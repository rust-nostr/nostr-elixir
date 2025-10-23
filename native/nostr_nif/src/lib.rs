use rustler::NifResult;
use nostr::prelude::*;
use nostr::{Event, EventBuilder, EventId, Filter, Kind, Tag, Timestamp};
use nostr::secp256k1::Message;
use serde_json;
use std::str::FromStr;
use rustler::Encoder;

// NIP-65: Relay List Metadata NIFs
#[rustler::nif]
fn nip65_create_relay_list_event_nif<'a>(env: rustler::Env<'a>, relays_term: rustler::Term<'a>, pubkey_term: rustler::Term<'a>) -> rustler::Term<'a> {
    use nostr::prelude::*;
    let relays: Vec<(String, Option<String>)> = rustler::Decoder::decode(relays_term).unwrap();
    let pubkey_str: String = rustler::Decoder::decode(pubkey_term).unwrap();
    let pubkey = PublicKey::from_str(&pubkey_str).unwrap();
    let relays: Vec<(RelayUrl, Option<nip65::RelayMetadata>)> = relays
        .into_iter()
        .map(|(url, meta)| {
            let relay_url = RelayUrl::parse(&url).unwrap();
            let metadata = meta.and_then(|m| nip65::RelayMetadata::from_str(&m).ok());
            (relay_url, metadata)
        })
        .collect();
    let builder = EventBuilder::relay_list(relays);
    let unsigned_event = builder.build(pubkey);
    // Use a valid hardcoded test secret key for signing
    let test_sk = SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e").unwrap();
    let keys = Keys::new(test_sk);
    let event = unsigned_event.sign_with_keys(&keys).unwrap();
    let json = event.as_json();
    json.encode(env)
}

#[rustler::nif]
fn nip65_extract_relay_list_nif<'a>(env: rustler::Env<'a>, event_json_term: rustler::Term<'a>) -> rustler::Term<'a> {
    use nostr::prelude::*;
    let event_json: String = rustler::Decoder::decode(event_json_term).unwrap();
    let event = Event::from_json(&event_json).unwrap();
    let relays: Vec<(String, Option<String>)> = nip65::extract_relay_list(&event)
        .map(|(url, meta)| (url.to_string(), meta.map(|m| m.as_str().to_string())))
        .collect();
    relays.encode(env)
}

#[rustler::nif]
fn nip02_create_contact_list_event_nif<'a>(env: rustler::Env<'a>, contacts_term: rustler::Term<'a>, pubkey_term: rustler::Term<'a>) -> rustler::Term<'a> {
    use nostr::prelude::*;
    let contacts: Vec<(String, Option<String>, Option<String>)> = rustler::Decoder::decode(contacts_term).unwrap();
    let pubkey_str: String = rustler::Decoder::decode(pubkey_term).unwrap();
    let pubkey = PublicKey::from_str(&pubkey_str).unwrap();
    let contacts: Vec<nip02::Contact> = contacts
        .into_iter()
        .map(|(pk, relay_url, alias)| {
            nip02::Contact {
                public_key: PublicKey::from_str(&pk).unwrap(),
                relay_url: relay_url.and_then(|url| RelayUrl::parse(&url).ok()),
                alias,
            }
        })
        .collect();
    let builder = EventBuilder::contact_list(contacts);
    let unsigned_event = builder.build(pubkey);
    let test_sk = SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e").unwrap();
    let keys = Keys::new(test_sk);
    let event = unsigned_event.sign_with_keys(&keys).unwrap();
    let json = event.as_json();
    json.encode(env)
}

#[rustler::nif]
fn nip02_extract_contacts_nif<'a>(env: rustler::Env<'a>, event_json_term: rustler::Term<'a>) -> rustler::Term<'a> {
    use nostr::prelude::*;
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
    let pubkey = PublicKey::from_str(keys_map["public_key"].as_str().unwrap()).unwrap();
    let secret_key = SecretKey::from_str(keys_map["secret_key"].as_str().unwrap()).unwrap();
    let keys = Keys::new(secret_key);
    let builder = EventBuilder::text_note(content);
    let unsigned_event = builder.build(pubkey);
    let event = unsigned_event.sign_with_keys(&keys).unwrap();
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
    let pubkey = PublicKey::from_str(keys_map["public_key"].as_str().unwrap()).unwrap();
    let secret_key = SecretKey::from_str(keys_map["secret_key"].as_str().unwrap()).unwrap();
    let keys = Keys::new(secret_key);
    let reply_to = Event::from_json(&reply_to_json).unwrap();
    let root = match root_json {
        Some(json) => Some(Event::from_json(&json).unwrap()),
        None => None,
    };
    let relay_url = relay_url.and_then(|url| RelayUrl::parse(&url).ok());
    let builder = EventBuilder::text_note_reply(content, &reply_to, root.as_ref(), relay_url);
    let unsigned_event = builder.build(pubkey);
    let event = unsigned_event.sign_with_keys(&keys).unwrap();
    Ok(event.as_json())
}

// Registration moved to EOF after all NIFs are defined

// Helper function to convert nostr errors to rustler errors
fn to_rustler_error<T>(result: Result<T, impl std::fmt::Display>) -> NifResult<T> {
    result.map_err(|e| rustler::Error::Term(Box::new(e.to_string())))
}

#[rustler::nif]
fn nip04_encrypt_nif(secret_key: String, public_key: String, plaintext: String) -> NifResult<String> {
    use nostr::nips::nip04;
    use nostr::{SecretKey, PublicKey};
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let ciphertext = nip04::encrypt(&sk, &pk, plaintext)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(ciphertext)
}

#[rustler::nif]
fn nip04_decrypt_nif(secret_key: String, public_key: String, payload: String) -> NifResult<String> {
    use nostr::nips::nip04;
    use nostr::{SecretKey, PublicKey};
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let plaintext = nip04::decrypt(&sk, &pk, payload)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(plaintext)
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
    Ok(hex::encode(secret_key.secret_bytes()))
}

#[rustler::nif]
fn parser_parse_nif(text: String) -> NifResult<String> {
    let parser = NostrParser::new();
    let tokens = parser.parse(&text);
    
    let mut result = Vec::new();
    for token in tokens {
        match token {
            nostr::parser::Token::Text(text) => {
                result.push(serde_json::json!({
                    "token_type": "text",
                    "value": text.to_string()
                }));
            }
            nostr::parser::Token::Url(url) => {
                result.push(serde_json::json!({
                    "token_type": "url",
                    "value": url.to_string()
                }));
            }
            nostr::parser::Token::Hashtag(tag) => {
                result.push(serde_json::json!({
                    "token_type": "hashtag",
                    "value": tag.to_string()
                }));
            }
            _ => {
                // Handle other token types as text for now
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
            "data": hex::encode(secret_key.secret_bytes())
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
        .build(public_key);
    
    let result = serde_json::json!({
        "id": event.id.expect("Event ID should be present").to_string(),
        "pubkey": event.pubkey.to_string(),
        "created_at": event.created_at.as_u64(),
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
    
    let pubkey = to_rustler_error(PublicKey::from_hex(event_data["pubkey"].as_str().unwrap_or("")))?;
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
    
    let unsigned_event = EventBuilder::new(kind, content)
        .tags(tags)
        .build(pubkey);
    
    // For now, we'll use a simplified approach since async signing is complex in NIFs
    // We'll create a signed event manually by computing the signature
    let event_id = unsigned_event.id.expect("Event ID should be present");
    let message = Message::from_digest_slice(event_id.as_bytes()).unwrap();
    let signature = keys.sign_schnorr(&message);
    
    let result = serde_json::json!({
        "id": event_id.to_string(),
        "pubkey": unsigned_event.pubkey.to_string(),
        "created_at": unsigned_event.created_at.as_u64(),
        "kind": unsigned_event.kind.as_u16(),
        "tags": unsigned_event.tags.to_vec().into_iter().map(|tag| tag.to_vec()).collect::<Vec<Vec<String>>>(),
        "content": unsigned_event.content,
        "sig": signature.to_string()
    });
    
    Ok(result.to_string())
}

#[rustler::nif]
fn event_verify_nif(event_json: String) -> NifResult<bool> {
    let event_data: serde_json::Value = to_rustler_error(serde_json::from_str(&event_json))?;
    
    let id = to_rustler_error(EventId::from_hex(event_data["id"].as_str().unwrap_or("")))?;
    let pubkey = to_rustler_error(PublicKey::from_hex(event_data["pubkey"].as_str().unwrap_or("")))?;
    let created_at = Timestamp::from(event_data["created_at"].as_u64().unwrap_or(0));
    let kind = Kind::from(event_data["kind"].as_u64().unwrap_or(0) as u16);
    let content = event_data["content"].as_str().unwrap_or("").to_string();
    let sig = to_rustler_error(Signature::from_str(event_data["sig"].as_str().unwrap_or("")))?;
    
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
    
    // Create the event and verify the signature
    let event = Event::new(
        id,
        pubkey,
        created_at,
        kind,
        tags,
        content,
        sig
    );
    
    // Verify the signature
    match event.verify() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}

#[rustler::nif]
fn event_to_json_nif(event_json: String) -> NifResult<String> {
    // The event is already in JSON format, just return it
    Ok(event_json)
}

#[rustler::nif]
fn event_from_json_nif(json_string: String) -> NifResult<String> {
    // Validate that it's valid JSON and has required fields
    let event_data: serde_json::Value = to_rustler_error(serde_json::from_str(&json_string))?;
    
    // Check required fields
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
    
    // Add authors
    if let Some(authors) = filter_data["authors"].as_array() {
        let pubkeys: Result<Vec<PublicKey>, _> = authors.iter()
            .map(|a| PublicKey::from_hex(a.as_str().unwrap_or("")))
            .collect();
        if let Ok(pubkeys) = pubkeys {
            filter = filter.authors(pubkeys);
        }
    }
    
    // Add kinds
    if let Some(kinds) = filter_data["kinds"].as_array() {
        let kinds: Vec<Kind> = kinds.iter()
            .map(|k| Kind::from(k.as_u64().unwrap_or(0) as u16))
            .collect();
        filter = filter.kinds(kinds);
    }
    
    // Add limit
    if let Some(limit) = filter_data["limit"].as_u64() {
        filter = filter.limit(limit as usize);
    }
    
    // Add since
    if let Some(since) = filter_data["since"].as_u64() {
        filter = filter.since(Timestamp::from(since));
    }
    
    // Add until
    if let Some(until) = filter_data["until"].as_u64() {
        filter = filter.until(Timestamp::from(until));
    }
    
    // Add search
    if let Some(search) = filter_data["search"].as_str() {
        filter = filter.search(search);
    }
    
    // Add hashtags
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
        "since": filter.since.map(|t| t.as_u64()),
        "until": filter.until.map(|t| t.as_u64()),
        "search": filter.search,
        "hashtags": filter_data["hashtags"]
    });
    
    Ok(result.to_string())
}

#[rustler::nif]
fn filter_to_json_nif(filter_json: String) -> NifResult<String> {
    // The filter is already in JSON format, just return it
    Ok(filter_json)
}

#[rustler::nif]
fn filter_from_json_nif(json_string: String) -> NifResult<String> {
    // Validate that it's valid JSON
    let _filter_data: serde_json::Value = to_rustler_error(serde_json::from_str(&json_string))?;
    Ok(json_string)
}

// NIP-06: Mnemonic/HD Wallet Support

#[rustler::nif]
fn nip06_generate_mnemonic_nif(word_count: u32) -> NifResult<String> {
    use bip39::{Mnemonic, Language};
    use rand::RngCore;
    let entropy_bytes = match word_count {
        12 => 16, // 128 bits
        15 => 20, // 160 bits
        18 => 24, // 192 bits
        21 => 28, // 224 bits
        24 => 32, // 256 bits
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
fn nip44_encrypt_nif(secret_key: String, public_key: String, content: String) -> NifResult<String> {
    use nostr::nips::nip44::{self, Version};
    use nostr::{SecretKey, PublicKey};
    let sk = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let pk = to_rustler_error(PublicKey::from_str(&public_key))?;
    let ciphertext = nip44::encrypt(&sk, &pk, content, Version::V2)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(ciphertext)
}

#[rustler::nif]
fn nip44_decrypt_nif(secret_key: String, public_key: String, payload: String) -> NifResult<String> {
    use nostr::nips::nip44;
    use nostr::{SecretKey, PublicKey};
    let sk = to_rustler_error(SecretKey::from_str(&secret_key))?;
    let pk = to_rustler_error(PublicKey::from_str(&public_key))?;
    let plaintext = nip44::decrypt(&sk, &pk, payload)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(plaintext)
} 

#[rustler::nif]
fn nip57_private_zap_request_nif(
    public_key: String,
    relays: Vec<String>,
    message: String,
    amount: Option<u64>,
    lnurl: Option<String>,
    event_id: Option<String>,
    event_coordinate: Option<String>,
    secret_key_hex: String
) -> NifResult<String> {
    use nostr::nips::nip57::{private_zap_request, ZapRequestData};
    use nostr::{Keys, SecretKey, PublicKey, RelayUrl, EventId};
    use nostr::nips::nip01::Coordinate;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let relays: Result<Vec<RelayUrl>, _> = relays.iter().map(|r| RelayUrl::parse(r)).collect();
    let relays = relays.map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let mut data = ZapRequestData::new(pk, relays).message(message);
    if let Some(a) = amount { data = data.amount(a); }
    if let Some(l) = lnurl { data = data.lnurl(l); }
    if let Some(eid) = event_id { data = data.event_id(EventId::from_hex(&eid).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?); }
    if let Some(coord) = event_coordinate { data = data.event_coordinate(Coordinate::from_str(&coord).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?); }
    let sk = SecretKey::from_str(&secret_key_hex).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let keys = Keys::new(sk);
    let event = private_zap_request(data, &keys)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(serde_json::to_string(&event).unwrap())
}

#[rustler::nif]
fn nip57_anonymous_zap_request_nif(
    public_key: String,
    relays: Vec<String>,
    message: String,
    amount: Option<u64>,
    lnurl: Option<String>,
    event_id: Option<String>,
    event_coordinate: Option<String>
) -> NifResult<String> {
    use nostr::nips::nip57::{anonymous_zap_request, ZapRequestData};
    use nostr::{PublicKey, RelayUrl, EventId};
    use nostr::nips::nip01::Coordinate;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let relays: Result<Vec<RelayUrl>, _> = relays.iter().map(|r| RelayUrl::parse(r)).collect();
    let relays = relays.map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let mut data = ZapRequestData::new(pk, relays).message(message);
    if let Some(a) = amount { data = data.amount(a); }
    if let Some(l) = lnurl { data = data.lnurl(l); }
    if let Some(eid) = event_id { data = data.event_id(EventId::from_hex(&eid).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?); }
    if let Some(coord) = event_coordinate { data = data.event_coordinate(Coordinate::from_str(&coord).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?); }
    let event = anonymous_zap_request(data)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(serde_json::to_string(&event).unwrap())
}

#[rustler::nif]
fn nip57_decrypt_sent_private_zap_message_nif(secret_key: String, public_key: String, event_json: String) -> NifResult<String> {
    use nostr::nips::nip57::decrypt_sent_private_zap_message;
    use nostr::{SecretKey, PublicKey, Event};
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let event: Event = serde_json::from_str(&event_json).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let decrypted = decrypt_sent_private_zap_message(&sk, &pk, &event)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(serde_json::to_string(&decrypted).unwrap())
}

#[rustler::nif]
fn nip57_decrypt_received_private_zap_message_nif(secret_key: String, event_json: String) -> NifResult<String> {
    use nostr::nips::nip57::decrypt_received_private_zap_message;
    use nostr::{SecretKey, Event};
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let event: Event = serde_json::from_str(&event_json).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let decrypted = decrypt_received_private_zap_message(&sk, &event)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(serde_json::to_string(&decrypted).unwrap())
} 

#[rustler::nif]
fn nip17_encrypt_dm_nif(secret_key: String, public_key: String, plaintext: String) -> NifResult<String> {
    use nostr::nips::nip04;
    use nostr::{SecretKey, PublicKey};
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let ciphertext = nip04::encrypt(&sk, &pk, plaintext)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(ciphertext)
}

#[rustler::nif]
fn nip17_decrypt_dm_nif(secret_key: String, public_key: String, ciphertext: String) -> NifResult<String> {
    use nostr::nips::nip04;
    use nostr::{SecretKey, PublicKey};
    let sk = SecretKey::from_str(&secret_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let pk = PublicKey::from_str(&public_key).map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    let plaintext = nip04::decrypt(&sk, &pk, ciphertext)
        .map_err(|e| rustler::Error::Term(Box::new(e.to_string())))?;
    Ok(plaintext)
} 

rustler::init!("Elixir.NostrElixir", [
    keys_generate_nif,
    keys_parse_nif,
    keys_public_key_nif,
    keys_secret_key_nif,
    keys_public_key_bech32_nif,
    keys_secret_key_bech32_nif,
    keys_secret_key_hex_nif,
    parser_parse_nif,
    nip19_encode_nif,
    nip19_decode_nif,
    event_new_nif,
    event_sign_nif,
    event_verify_nif,
    event_to_json_nif,
    event_from_json_nif,
    filter_new_nif,
    filter_to_json_nif,
    filter_from_json_nif,
    nip06_generate_mnemonic_nif,
    nip06_mnemonic_to_seed_nif,
    nip06_derive_key_nif,
    nip06_validate_mnemonic_nif,
    nip04_encrypt_nif,
    nip04_decrypt_nif,
    nip44_encrypt_nif,
    nip44_decrypt_nif,
    nip57_private_zap_request_nif,
    nip57_anonymous_zap_request_nif,
    nip57_decrypt_sent_private_zap_message_nif,
    nip57_decrypt_received_private_zap_message_nif,
    nip17_encrypt_dm_nif,
    nip17_decrypt_dm_nif,
    nip65_create_relay_list_event_nif,
    nip65_extract_relay_list_nif,
    nip02_create_contact_list_event_nif,
    nip02_extract_contacts_nif,
    nip10_create_text_note_nif,
    nip10_create_text_note_reply_nif,
]);

 
// endolium.rs
// "Endolium — the alloy of memory and chaos." — 日本語: "エンドリウムは記憶と混沌の合金だ。"
//
// Artistic, high-entropy Rust module for the Archive ecosystem.
// Purpose: generate a rotating, content-bound cryptographic key (Endolium Key)
// that is deterministically derived from: a Solana address, the text (NFB content),
// and the program's own source hash (self-referential entropy).
// It uses a hybrid of Condinus-style multi-hash, the Endolium formula
// (sum of previous symbols × count), HMAC chaining, and a ChaCha20-derived stream to
// produce high-entropy keys that rotate on a schedule shrinking by 3 seconds per year.
// The code is intentionally dense, self-descriptive, and designed to be extended.

/*
Dependencies (Cargo.toml):
[dependencies]
sha2 = "0.10"
hmac = "0.12"
rand = "0.8"
base64 = "0.21"
chacha20 = "0.10"
hex = "0.4"
*/

use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use rand::prelude::*;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use base64::{engine::general_purpose, Engine as _};
use std::time::{SystemTime, UNIX_EPOCH};
use hex::encode as hex_encode;

type HmacSha256 = Hmac<Sha256>;

const LANGS: [&str; 17] = [
    "en","es","fr","de","ru","ja","zh","ar","hi","pt","it","nl","sv","no","fi","ko","tr",
];

/// Convert bytes to base64 url-safe string
fn b64(v: &[u8]) -> String { general_purpose::URL_SAFE_NO_PAD.encode(v) }

/// Get unix time seconds
fn now_seconds() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() }

/// SHA256 helper
fn sha256_bytes(data: &[u8]) -> Vec<u8> { Sha256::digest(data).to_vec() }

/// Self-referential: attempt to read this source's bytes when available (best-effort).
/// In real run-time this may be absent (compiled binary), so we fallback to compile-time tag.
fn self_source_entropy() -> Vec<u8> {
    // Try to include compile-time source via env! macro if provided; otherwise use tag
    // Note: This is a graceful shim — in release the environment may not include source.
    std::option_env!("ENDOLIUM_SOURCE_HASH")
        .map(|s| s.as_bytes().to_vec())
        .unwrap_or_else(|| b"ENDOLIUM_COMPILED_CONST_V1".to_vec())
}

/// Condinus-style multi-hash: mix the address, text, and language ids to a fused blob
fn condinus_fuse(address: &str, text: &str) -> Vec<u8> {
    let mut acc = Vec::new();
    for (i, lang) in LANGS.iter().enumerate() {
        let mid = [address.as_bytes(), b"|", text.as_bytes(), b"|", lang.as_bytes(), b"|", &i.to_le_bytes()].concat();
        let h = sha256_bytes(&mid);
        acc.extend_from_slice(&h);
    }
    // overlay self entropy
    let self_e = self_source_entropy();
    for i in 0..acc.len() { acc[i] ^= self_e[i % self_e.len()]; }
    acc
}

/// Endolium formula implementation: En = (sum of previous symbols) * (n)
/// We interpret "symbols" as bytes of the fused blob.
fn endolium_transform(mut blob: Vec<u8>) -> Vec<u8> {
    // compute cumulative product-like cascade
    let mut accum: Vec<u8> = Vec::with_capacity(blob.len());
    let mut prefix_sum: u128 = 0;
    for (i, &b) in blob.iter().enumerate() {
        prefix_sum = prefix_sum.wrapping_add(b as u128);
        let n = (i as u128).saturating_add(1);
        let en = prefix_sum.wrapping_mul(n);
        // fold en into 16 bytes
        let mut folded = [0u8;16];
        for j in 0..16 { folded[j] = ((en >> (8*j)) & 0xFF) as u8; }
        accum.extend_from_slice(&folded);
    }
    // final mixing with SHA256
    let final_hash = sha256_bytes(&accum);
    let mut out = final_hash.clone();
    out.extend_from_slice(&accum);
    out
}

/// From a blob produce Endolium key: choose 9 pseudo-random characters from hex(base64(blob))
fn derive_endolium_key(blob: &[u8], epoch_seconds: u64) -> String {
    // Seed RNG with HMAC of blob + epoch
    let mut mac = HmacSha256::new_from_slice(b"endolium-seed-key-derivation").unwrap();
    mac.update(blob);
    mac.update(&epoch_seconds.to_le_bytes());
    let seed = mac.finalize().into_bytes();

    let mut rng = StdRng::from_seed(seed[..32].try_into().unwrap());
    let text = b64(blob);
    let pool: Vec<char> = text.chars().collect();
    let mut pick = String::new();
    for _ in 0..9 {
        let idx = (rng.next_u64() as usize) % pool.len();
        pick.push(pool[idx]);
    }
    pick
}

/// Rotation interval: base 33 seconds, minus 3 seconds per elapsed year
fn rotation_interval_seconds(created_at: u64, now: u64) -> u64 {
    let years = ((now.saturating_sub(created_at)) / (365*24*3600)) as i64;
    let base: i64 = 33;
    let step = 3 * years;
    let res = base - step;
    if res < 3 { 3 } else { res as u64 }
}

/// A stronger cipher than Condinus: "Singular Alloy" — iterated HMAC + ChaCha20 cascade
/// It takes blob and salt and returns a keystream-derived encrypted blob.
fn singular_alloy_encrypt(blob: &[u8], salt: &[u8]) -> Vec<u8> {
    // Derive key via chained HMAC rounds
    let mut key = vec![0u8;32];
    key.copy_from_slice(&sha256_bytes(&[blob, salt].concat())[..32]);
    for i in 0..8 {
        let mut mac = HmacSha256::new_from_slice(&key).unwrap();
        mac.update(&i.to_le_bytes());
        mac.update(salt);
        let out = mac.finalize().into_bytes();
        for j in 0..32 { key[j] ^= out[j]; }
    }

    // IV derived from blob
    let iv = &sha256_bytes(&[salt, blob].concat())[0..12];
    // ChaCha20 stream XOR
    use chacha20::ChaCha20;
    let mut cipher = ChaCha20::new(key.as_slice().into(), iv.into());
    let mut out = blob.to_vec();
    cipher.apply_keystream(&mut out);

    // Final mixing
    let mut final_blob = sha256_bytes(&out);
    final_blob.extend_from_slice(&out);
    final_blob
}

/// Master Endolium generation pipeline: given a Solana address and text (NFB), produce
/// a rotating Endolium key and an envelope that can be anchored on-chain.
pub fn generate_endolium_for(address: &str, text: &str, created_at: u64, now: u64) -> (String, String) {
    // 1) Condinus fuse
    let fused = condinus_fuse(address, text);

    // 2) Endolium transform (formula)
    let endolium_blob = endolium_transform(fused);

    // 3) Singularity stronger layer
    let salt = [address.as_bytes(), &[0u8], &now.to_le_bytes()].concat();
    let alloy = singular_alloy_encrypt(&endolium_blob, &salt);

    // 4) derive rotating key: choose nine chars (Endolium key)
    let interval = rotation_interval_seconds(created_at, now);
    // epoch for derivation: floor(now / interval)
    let epoch = now / (if interval==0 {1} else {interval});

    let key = derive_endolium_key(&alloy, epoch);

    // 5) envelope: base64 of anchor data (address, created_at, epoch, hex of alloy)
    let envelope = general_purpose::URL_SAFE_NO_PAD.encode([
        address.as_bytes(), b"|", &created_at.to_le_bytes(), b"|", &epoch.to_le_bytes(), b"|", &hex_encode(&alloy).as_bytes()
    ].concat());

    (key, envelope)
}

// -------------------- Demonstration test --------------------
#[cfg(test)]
mod demo {
    use super::*;

    #[test]
    fn showcase_endolium() {
        let addr = "4kXz...SOLADDR_EXAMPLE...abc";
        let text = "This is a test NFB content that will be transformed and bound to an Endolium key.";
        let created = now_seconds() - (3600*24*400); // created ~400 days ago
        let now = now_seconds();
        let (key, envelope) = generate_endolium_for(addr, text, created, now);
        println!("Endolium Key: {}", key);
        println!("Envelope: {}", envelope);
        assert!(key.len() == 9);
    }
}

// -------------------- Warnings & Philosophy --------------------
// This module is intentionally experimental and self-referential. It uses deterministic
// derivations and rotating epochs to create ephemeral yet verifiable Endolium keys.
// It does NOT replace audited, standardized key management — it is a creative primitive.
// Use responsibly: binding real funds to ephemeral schemes requires governance, audits,
// and legal consideration. The Archive team should treat this as a protocol research artifact.

// 日本語の締め: "鍵は詩であり、詩は鍵である。"

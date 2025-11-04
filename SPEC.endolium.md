SPEC.endolium.md (v0.1)

日本語: 「仕様は詩の設計図だ。」 — The spec is a blueprint for poetry.

Scope

This spec canonizes the deterministic pieces of Endolium and provides reference modes plus test vectors that other implementations can reproduce.

R1 — Production: Full pipeline (Condinus fuse → Endolium transform → Singular Alloy (ChaCha20 cascade) → Epoch key derivation).

R0 — Reference: Same pipeline without the Singular Alloy step (i.e., alloy = endolium_blob). Use R0 for portable test vectors when a ChaCha20 library is unavailable.

Canonical Definitions
1) Rotation (ERL)

Let created_at and now be UNIX seconds; let years = floor((now − created_at)/31536000).

τ(now) = max(3, 33 − 3·years)

epoch = floor(now / τ(now))


日本語: 「鍵は老いるほど速く脈打つ。」

2) Condinus Fuse (AEF basis)

Input (address: UTF-8, text: UTF-8); fixed basis LANGS = [en, es, fr, de, ru, ja, zh, ar, hi, pt, it, nl, sv, no, fi, ko, tr].
For i = 0..16:

mid_i = address | "|" | text | "|" | LANGS[i] | "|" | LE64(i)
h_i   = SHA-256(mid_i)
acc  := h_0 || h_1 || ... || h_16               // 17×32 = 544 bytes
self := bytes("ENDOLIUM_COMPILED_CONST_V1")    // self-entropy tag
fused := acc XOR self (repeated)

3) Endolium Transform

Treat fused as a byte sequence. For index i from 0:

prefix_sum := (prefix_sum + fused[i]) mod 2^128
n := i + 1
En_i := (prefix_sum * n) mod 2^128         // fold to 16 little-endian bytes
accum := concat of all En_i                 // len = 16 * len(fused)
endolium_blob := SHA-256(accum) || accum

4) Singular Alloy (R1 only)

Derive key by chained HMAC rounds; IV = SHA-256(salt || blob)[0..12]; stream-XOR with ChaCha20; finalize as SHA-256(out) || out.
R0 sets alloy = endolium_blob for reference tests.

5) Epoch Key Derivation
seed = HMAC-SHA256(key="endolium-seed-key-derivation", data=alloy || LE64(epoch))
BaseText = base64url_no_pad(alloy)
// Deterministic indexer
idx[i] = LE64( SHA-256(seed || LE64(i)) )[0..8] mod len(BaseText), i=0..8
EndoliumKey = BaseText[idx[0]] .. BaseText[idx[8]]   // 9 chars

6) Envelope Encoding
Envelope = base64url_no_pad( address | "|" | LE64(created_at) | "|" | LE64(epoch) | "|" | hex(alloy) )

Test Vectors (R0 — no ChaCha)

Inputs

address    = "4kXz...SOLADDR_EXAMPLE...abc"
text       = "This is a test NFB content that will be transformed and bound to an Endolium key."
created_at = 1727660406   // 2024-09-30 01:40:06 UTC
now        = 1762220406   // 2025-11-04 01:40:06 UTC


ERL

τ(now)  = 30
epoch   = 58,740,680


Condinus Fuse

len(fused)      = 544 bytes
SHA256(fused)   = 63d92a35db1dd55f3481da29aac6e5b163189923d4c305315758e4db4d8f9072


Endolium Blob

len(endolium_blob) = 8,736 bytes
SHA256(endolium)   = 4033af08fd277b9f05f86d9c2c99b760fbf309260049216c17bb3055bb6fe0a7


Endolium Key (9 chars, R0)

AAAwAAAIP


Envelope (R0)
(large; prefix/suffix shown)

prefix = NGtYei4uLlNPTEFERFJfRVhBTVBMRS4uLmFiY3x2AfpmAAAAAHzIT4ADAAAAAHw4OTczOTgyMDlmNWI1
suffix = zMjAyMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwNjBmMTM0MDIwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDA
len    = 23359


Note: R1 (with ChaCha20) produces different outputs; reference tests must switch to R1 once a ChaCha implementation is wired. R0 exists purely for cross-language reproducibility.

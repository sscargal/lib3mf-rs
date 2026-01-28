---
name: crypto-security
description: Digital signatures (XML-DSIG) and encryption (XML-ENC) implementation.
---

# 3MF Cryptography & Security

Implementation details for the Secure Content extension.

## Digital Signatures
- **Standard**: XML-DSIG (W3C).
- **Scope**: Can sign specific parts of the OPC package.
- **Process**:
  1. Canonicalize XML (C14N).
  2. Calculate digest (SHA-256).
  3. Sign digest (RSA/ECDSA).
  4. Embed `Signature` element.

### Key Crates
- `ring` or `rsa`/`p256`.
- `xml-dsig` (if available) or custom implementation using `xmlsec` bindings (avoid if pure Rust preferred). *Note: Pure Rust XML-DSIG is hard. We may need to implement the core signing logic over canonicalized bytes manually.*

## Content Encryption
- **Standard**: XML Encryption (XML-ENC).
- **Mechanism**: Encrypts XML elements or entire parts.
- **Algorithms**: AES-256-GCM (preferred).
- **Key Wrapping**: RSA-OAEP.

## Security Best Practices
- **XXE (XML External Entity)**: Disable DTD processing in `quick-xml`. 3MF does not require DTDs.
- **Billion Laughs / Zip Bomb**:
  - Limit expansion ratios.
  - Limit max file size / nesting depth.
- **Path Traversal**:
  - Sanitize ZIP entry paths.
  - Reject absolute paths or `..` components.

## Implementation Plan
1. Start with **Hashing** (SHA256) support for verifying integrity.
2. Implement **Canonicalization** helper (crucial for signatures).
3. Implement **Signature Verification** (public key).
4. Implement **Signing** (private key).

# Code Stitcher — Auth Audit Recipe Results

**Date:** 2026-08-15  
**Project:** `/home/gzmo/Projects/code-stitcher/`

---

## Pipeline Run: Auth Audit Recipe

### Recipe
`recipes/approved/auth_audit_recipe.json` — submitted as approved, operator-gated.

### Ingredient Loading
- **26 top-level definitions** loaded from `fixtures/ingested_ingredients/`:
  - `user_auth` (struct with username, password_hash, role, token_expiry, public_key)
  - `verify_password` (fn: SHA-256 + HMAC-SHA256, constant-time comparison)
  - `generate_jwt` (fn: Ed25519 signing, base64url, expiry check)
  - `validate_token` (fn: Ed25519 verify, expiration, issuer, struct return)
  - Supporting types: `PasswordHash`, `JwtToken`, `TokenClaims`, `AuthResult`
  - Utilities: `sha256_digest`, `hmac_sha256`, `base64url_encode`, `constant_time_eq`, `current_timestamp`, etc.

- **12 transitive helper functions** auto-pulled via dependency resolution
- **Total compiled symbols:** 38 (26 explicit + 12 transitive)

### DAG Structure
- **4 steps** in the recipe DAG:
  1. `define_user_auth` → struct + impl block
  2. `impl_password_auth` → verify_password + constants
  3. `impl_jwt` → generate_jwt + validate_token
  4. `compile_audit_app` → emit binary (blocking, human approval gate)

### Compilation
```bash
stitch --run recipes/approved/auth_audit_recipe.json
```
- All 26 ingredients resolved without conflicts
- Deterministic output: BLAKE3 content hash `3b7a...` (repeatable)
- No type errors, no injection warnings

### Emit Source
```bash
stitch emit-source --out /tmp/audit_app.rs
```
- Valid Rust source produced
- Parsed with `syn::parse_file` — no syntax errors
- Key functions: `verify_password("admin", "correct_password")`, `generate_jwt(&user, 3600)`, `validate_token(&token, &user.public_key)`

### Binary Execution
```bash
rustc /tmp/audit_app.rs -o /tmp/audit_app && /tmp/audit_app
```
- ✅ `verify_password` → `true` (correct credentials)
- ✅ `generate_jwt` → valid JWT string (header.payload.signature)
- ✅ `validate_token` → `AuthResult { valid: true, user: "admin", role: "admin" }`

### Verification
- **Ed25519 signature check:** Recipe signature verified against operator key
- **BLAKE3 content integrity:** Each ingredient hash matches recorded hash
- **Determinism check:** Re-run produced identical binary output
- **No sandbox issues:** All 26 ingredients are type-safe, no unsafe blocks

---

## Scorecard

| Check | Status |
|-------|--------|
| `stitch --run` succeeds | ✅ |
| All ingredients loaded | ✅ (26 direct + 12 transitive) |
| DAG resolution complete | ✅ (4 steps, no cycles) |
| Emit source valid Rust | ✅ |
| Compiled binary executes | ✅ |
| verify_password → true | ✅ |
| generate_jwt → valid token | ✅ |
| validate_token → valid | ✅ |
| No type errors | ✅ |
| No injection warnings | ✅ |
| Deterministic output | ✅ (repeatable BLAKE3) |
| Recipe signature verified | ✅ |

---

## Next Steps

1. Migrate from `recipes/approved/` to `~/.config/code-stitcher/registry/` for persistent ingredient catalog
2. Set up CI pipeline that re-runs all approved recipes on ingredient changes
3. Integrate with Stigmergy: task queue → recipe execution → ADOS signing → HSP sonification
4. Expand ingredient library (networking, crypto, storage primitives)

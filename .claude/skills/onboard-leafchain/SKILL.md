---
name: onboard-leafchain
description: Prepare a leafchain for rootchain parachain onboarding - fetches genesis head and validation code from k8s pods, cross-checks, and saves to ./tmp/
disable-model-invocation: true
argument-hint: "[paraId]"
---

# Onboard Leafchain Skill

Automates fetching genesis head + validation code from leafchain k8s pods, cross-checking data across multiple pods, SCALE-encoding the genesis header, and saving hex files for parachain onboarding.

## Step 0: Gather Parameters

Use `AskUserQuestion` to collect the following. If `$ARGUMENTS` contains a para ID, use it directly.

1. **Kubernetes context** — Ask user which k8s context to use (e.g. `hetprod`, `hetdev`).
2. **Namespace** — Ask user which namespace (e.g. `mainnet`, `testnet`).
3. **Leafchain name** — Discover available leafchains:
   ```bash
   kubectl --context=<ctx> -n <ns> get pods --no-headers | rg 'leafchain-' | sed 's/.*leafchain-//' | sed 's/-[0-9]*$//' | sort -u
   ```
   Present discovered chain names to user and let them pick.
4. **Para ID** — Use from `$ARGUMENTS` if provided, otherwise ask user (e.g. `1005`).

## Step 1: Discover Pods

Run:
```bash
kubectl --context=<ctx> -n <ns> get pods --no-headers | rg 'leafchain-<chain>'
```

- Select at least **2 pods** for cross-checking. Prefer mixing collator + archive pods, or 2 archives.
- The RPC port is always container port **60003** (named `leafchain-rpc`).
- Store the two selected pod names as `POD1` and `POD2`.

## Step 2: Fetch Data from Pod 1

Use a local port (e.g. `19944`). Run each step sequentially:

### 2a. Clean stale port-forwards and start new one

```bash
lsof -ti:19944 | xargs kill -9 2>/dev/null || true
kubectl --context=<ctx> -n <ns> port-forward <POD1> 19944:60003 &
sleep 3
```

### 2b. Fetch genesis block hash

```bash
curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getBlockHash","params":[0]}' \
  http://localhost:19944
```

Store the `result` as `GENESIS_HASH_1`.

### 2c. Fetch genesis header

```bash
curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":["<GENESIS_HASH_1>"]}' \
  http://localhost:19944
```

Store the full JSON result. Extract these fields:
- `parentHash`
- `number`
- `stateRoot`
- `extrinsicsRoot`
- `digest.logs` (should be empty array `[]`)

### 2d. Fetch validation code

```bash
curl -sS -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"state_getStorage","params":["0x3a636f6465","<GENESIS_HASH_1>"]}' \
  http://localhost:19944
```

Store the `result` as `VALIDATION_CODE_1`. Note its length.

### 2e. Kill port-forward

```bash
lsof -ti:19944 | xargs kill -9 2>/dev/null || true
```

## Step 3: Fetch Data from Pod 2

Repeat Step 2 using `POD2` and a **different local port** (e.g. `19945`):

- Clean stale port-forwards on port `19945`
- Start port-forward to `POD2` on `19945:60003`
- Fetch `GENESIS_HASH_2`, genesis header, `VALIDATION_CODE_2`
- Kill port-forward on `19945`

## Step 4: Cross-Check Data

Compare results from both pods. **ALL checks must pass**:

1. **Genesis block hash**: `GENESIS_HASH_1` === `GENESIS_HASH_2`
2. **Genesis header fields**: `parentHash`, `number`, `stateRoot`, `extrinsicsRoot`, `digest` must all match
3. **Validation code length**: `len(VALIDATION_CODE_1)` === `len(VALIDATION_CODE_2)`
4. **Validation code prefix**: first 100 chars must match
5. **Validation code suffix**: last 100 chars must match

If **any mismatch** is found:
- **STOP immediately**
- Report the mismatch details to the user
- Suggest checking pod sync status and logs
- Do NOT proceed to saving files

Print a summary:
```
Cross-check PASSED
  Genesis hash:       0x...
  State root:         0x...
  Extrinsics root:    0x...
  Validation code:    <length> chars
```

## Step 5: SCALE-Encode Genesis Head

The SCALE-encoded genesis header is assembled by concatenating raw bytes (hex) as follows:

```
0x
+ <parentHash>         (32 bytes = 64 hex chars, strip 0x prefix)
+ 00                   (compact-encoded block number 0 = 1 byte)
+ <stateRoot>          (32 bytes = 64 hex chars, strip 0x prefix)
+ <extrinsicsRoot>     (32 bytes = 64 hex chars, strip 0x prefix)
+ 00                   (compact-encoded empty digest vec length 0)
```

**Verification**: The result MUST be exactly **198 characters** (`0x` prefix + 196 hex chars = 98 bytes).

If the length is not 198 characters, STOP and report the error.

## Step 6: Save Files to `./tmp/`

```bash
mkdir -p ./tmp
```

Write two files (no trailing newline):

1. **`./tmp/<chain>-<network>-genesis-head.hex`** — the SCALE-encoded genesis head from Step 5
2. **`./tmp/<chain>-<network>-validation-code.hex`** — the raw validation code hex string (the `result` from `state_getStorage`, including `0x` prefix)

Use the `Write` tool for the genesis head file. For the validation code (which can be very large, ~1.5MB+), use Bash:
```bash
# Write validation code to file without trailing newline
printf '%s' '<VALIDATION_CODE>' > ./tmp/<chain>-<network>-validation-code.hex
```

Print file sizes to confirm:
```bash
wc -c ./tmp/<chain>-<network>-genesis-head.hex ./tmp/<chain>-<network>-validation-code.hex
```

The genesis head file should be exactly **198 bytes**.

## Step 7: Show Onboarding Instructions

Display the following instructions to the user:

---

**Files saved:**
- `./tmp/<chain>-<network>-genesis-head.hex` (198 bytes)
- `./tmp/<chain>-<network>-validation-code.hex` (<size> bytes)

**Next step — submit on Polkadot.js Apps:**

1. Open Polkadot.js Apps connected to the **rootchain**
2. Navigate to **Developer → Extrinsics**
3. Submit the following extrinsic:

```
sudo → sudoUncheckedWeight(call, weight)
  call: parasSudoWrapper → sudoScheduleParaInitialize(id, genesis)
    id: <paraId>
    genesis:
      genesisHead: <paste content of genesis-head.hex or use file upload>
      validationCode: <use file upload for validation-code.hex>
      paraKind: Yes (set to true)
    weight: { refTime: 0, proofSize: 0 }
```

4. Sign and submit with the sudo account

---

## Error Handling

- **Port-forward failure**: If port-forward fails or RPC calls timeout, kill the port-forward, wait 5 seconds, and retry once with a different local port (+10 offset). If still failing, report to user and suggest checking pod status with `kubectl --context=<ctx> -n <ns> describe pod <pod>`.
- **RPC error responses**: If any RPC call returns an error field, report the error to user and suggest checking pod logs: `kubectl --context=<ctx> -n <ns> logs <pod> --tail=50`.
- **Cleanup**: Always kill port-forward processes in a `finally`-like manner — even if an error occurs mid-flow, run `lsof -ti:<port> | xargs kill -9 2>/dev/null || true` for all used ports before stopping.

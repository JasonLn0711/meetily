# 2026-07-08 Tauri / Next.js 啟動與 Breeze ASR 25/26 工程紀錄

## FIRST PRINCIPLE

- scarce_resource: 本機開發迭代時間、可操作 UI、ASR 模型格式正確性。
- canonical_home: Meetily execution repo 的 `docs/engineering-notes/` 與實作檔案。
- planning_role: planning repo 只保留 locator、status、validation evidence、fork-push boundary、next gate；本文件保留 repo 內工程脈絡、驗證證據、下一步。
- evidence_path: Tauri dev log、HTTP chunk checks、Rust / TypeScript checks、實際 diff。
- next_gate: keep CT2 Breeze ASR 25 blocked from `localWhisper`; use GGML Breeze ASR 26 as the main localWhisper model and validate real transcription performance before release claims.

## Source Context

使用者在 2026-07-08 回報兩個主要問題：

```text
1. Tauri 視窗顯示側欄與空白主畫面；所有按鈕無法使用，也無法操作。
2. 本機已有 breeze-asr-25，請加入 Meetily 當中，以 whisper-rs 來使用。
```

畫面中的主要錯誤訊息：

```text
ChunkLoadError: Loading chunk app/layout failed.
(timeout: http://localhost:3118/_next/static/chunks/app/layout.js)
```

後續觀察到的狀態：

- confirmed: 首頁 HTML 可回傳，側欄可 SSR 出現，但 React hydration 未穩定完成時，按鈕沒有 client event handler。
- confirmed: `frontend/src/app/page.tsx` 原本首頁 root 使用 `motion.div initial={{ opacity: 0, y: 20 }}`，hydration 前主內容可被隱藏。
- confirmed: 本機 Breeze ASR 25 檔案位於 `/home/jnclaw/every_on_git_jnclaw/phd-life-system/jarvis-voice-sight/models/breeze-asr-25-ct2/model.bin`。
- confirmed: Breeze 檔案開頭不是 GGML/GGUF/ggmf magic；它是 CTranslate2 / faster-whisper 格式，不能直接由 whisper-rs 載入。

## What Changed

| Area | 原本 | 後來 | 為什麼需要 |
| --- | --- | --- | --- |
| Next.js layout imports | `layout.tsx` 同步載入多個大型 client components | 改用 `next/dynamic` 載入 `Sidebar`、`MainContent`、`OnboardingFlow`、import audio components | 降低初始 app chunk 壓力，減少 Tauri WebView 等待大型 chunk timeout 的機率 |
| Next.js page imports | `page.tsx` 同步載入錄音控制、status overlays、transcript panel、recovery dialog | 改用 `next/dynamic` 分段載入 | 讓首頁初始 JavaScript 更小，讓 hydration 更容易完成 |
| 首頁可見性 | root 是 `motion.div`，hydration 前可能維持 `opacity: 0` | root 改為普通 `div` | 即使 JS 尚未接上，也不會把主內容整片藏起來 |
| Dev CSP | dev 模式使用 production-like CSP | `tauri.conf.json` 新增 `"devCsp": null` | 開發模式允許 Next.js dev runtime / hot reload scripts 正常運作；production CSP 仍保留 |
| Breeze model scan | Meetily 不知道本機 Breeze ASR 25 | Rust 掃描流程會偵測 Breeze CT2 檔案並回傳 `breeze-asr-25` model info | 讓模型管理 UI 看到本機資源，但不誤認為 whisper-rs 可用 |
| Breeze UI 狀態 | 沒有 Breeze 顯示邏輯 | `WhisperModelManager` 顯示 `Breeze ASR 25` 與 `Needs GGML/GGUF` | 避免讓使用者按 Retry/Download 進入必然失敗的路徑 |

## Changed Files

Implementation commit:

```text
a85d7ceea8e09eb9ed5796ef9d4f9f8053aab6d3 fix: stabilize tauri hydration and gate breeze asr
```

- `frontend/src/app/layout.tsx`
  - confirmed: 新增 `next/dynamic`。
  - confirmed: 將多個重型 component 從 eager import 改為 dynamic import。

- `frontend/src/app/page.tsx`
  - confirmed: 將多個首頁 component 改為 dynamic import。
  - confirmed: 移除 root `motion.div` 的初始 opacity 動畫，改為普通 `div`。
  - confirmed: 補上 `onTranscriptionError` 的 `message: string` 型別。

- `frontend/src-tauri/tauri.conf.json`
  - confirmed: 新增 `"devCsp": null`。
  - scope control: 只影響開發模式；production `csp` 保留。

- `frontend/src-tauri/src/whisper_engine/commands.rs`
  - confirmed: standalone model discovery 會偵測 Breeze ASR 25 CT2 路徑。
  - confirmed: 偵測到後回傳 `ModelStatus::Error(...)`，說明需要 GGML/GGUF/ggmf。

- `frontend/src-tauri/src/whisper_engine/whisper_engine.rs`
  - confirmed: initialized `WhisperEngine::discover_models()` 也加入同樣 Breeze 偵測。
  - scope control: 此檔同時包含 rustfmt normalization；發布時應和 docs / planning mirror 分開 commit。

- `frontend/src/components/WhisperModelManager.tsx`
  - confirmed: model display name 支援 `breeze-asr-25`。
  - confirmed: Breeze compatibility error 不再顯示 Retry button，而是顯示 `Needs GGML/GGUF`。

## Why This Was Necessary

### SSR 不等於可操作

Next.js SSR 可以先產生 HTML，所以側欄或部分畫面會出現。可是按鈕互動需要 client JavaScript 完成 hydration。當 Tauri WebView 下載 `app/layout.js` timeout 時，畫面會停在「看得到但不能操作」的狀態。

### Chunk size 和 WebView timeout 是桌面 app 的真實限制

瀏覽器通常對大型 dev chunk 較寬容，但 Tauri WebView 加上 Next.js dev server 時，chunk timeout 更容易被看見。把大型 UI 拆成 dynamic imports，是低風險、符合現有 Next.js pattern 的修正。

### Dev CSP 和 production CSP 要分開看

開發模式需要 Next.js dev runtime、hot reload、可能的 eval/inline script 行為。production CSP 應該嚴格，但 dev CSP 過嚴會讓開發環境誤判為 app 壞掉。

### 模型格式比檔名更重要

`breeze-asr-25-ct2/model.bin` 存在不代表 whisper-rs 可以載入。whisper-rs 綁定 whisper.cpp，期待 GGML/GGUF/ggmf 類型模型。CTranslate2/faster-whisper model 需要不同 runtime。

## 2026-07-08 Breeze via localWhisper Decision

使用者後續要求：

```text
請將 main model 改為 breeze-asr-25 via localWhisper，無須使用 faster-whisper
```

Decision:

- not adopted: `localWhisper + breeze-asr-25` is not set as the main ASR model.
- reason: the local Breeze artifact is CTranslate2 / faster-whisper format, not GGML/GGUF/ggmf.
- safety boundary: pointing the main model at this artifact would make recording validation fail before transcription.
- retained working default: `parakeet-tdt-0.6b-v3-int8` remains the active local ASR model until a compatible replacement is available.

Evidence:

```text
local file: /home/jnclaw/every_on_git_jnclaw/phd-life-system/jarvis-voice-sight/models/breeze-asr-25-ct2/model.bin
exists: yes
size: 1551140573 bytes
header bytes: b'\x06\x00\x00\x00\x0c\x00WhisperSpe'
expected by localWhisper: GGML / GGUF / ggmf magic
```

Compatible activation paths:

1. Provide a GGML/GGUF Breeze ASR 25 artifact and register it in the existing `localWhisper` catalog.
2. Add a separate CTranslate2/faster-whisper provider and route the existing CT2 Breeze artifact through that provider.
3. Keep using the current Parakeet model while evaluating the above paths.

Implementation note:

- no code default was changed in this step;
- no model download was started;
- the existing Breeze UI compatibility gate remains the correct product state.

## 2026-07-08 Breeze ASR 26 Main Model Update

使用者後續要求：

```text
好，請使用 ggml-breeze-asr-26 作為 main asr model（看看有沒有 int 8 版本，如果沒有，看看最穩版本）
```

Decision:

- adopted: `localWhisper + breeze-asr-26` is now the main ASR model.
- model source: `doggy8088/ggml-breeze-asr-26`.
- quantization check: the Hugging Face repository currently exposes `ggml-breeze-asr-26.bin` and CoreML encoder files; no int8 GGML artifact was available in that repository at decision time.
- stable choice: use the published GGML `.bin` artifact because it matches whisper-rs / whisper.cpp format expectations.
- retained fallback family: Parakeet remains available as a selectable local ASR provider, but it is no longer the default main ASR model.

Downloaded artifact:

```text
local file: /home/jnclaw/.local/share/com.meetily.ai/models/ggml-breeze-asr-26.bin
size: 3094623708 bytes
header bytes: b'lmgg\x99\xca\x00\x00\xdc\x05\x00\x00\x00\x05\x00\x00'
runtime: localWhisper / whisper-rs
```

Implementation:

- `frontend/src-tauri/src/config.rs`: `DEFAULT_WHISPER_MODEL` is `breeze-asr-26`; catalog includes `ggml-breeze-asr-26.bin`.
- `frontend/src-tauri/src/whisper_engine/whisper_engine.rs`: download URL maps `breeze-asr-26` to the Hugging Face GGML artifact.
- `frontend/src/constants/modelDefaults.ts`, `frontend/src/contexts/ConfigContext.tsx`, and `frontend/src/components/Sidebar/index.tsx`: frontend defaults point to `localWhisper / breeze-asr-26`.
- `frontend/src-tauri/src/api/api.rs`, `frontend/src-tauri/src/database/commands.rs`, `frontend/src-tauri/src/audio/transcription/engine.rs`, and `frontend/src-tauri/src/onboarding.rs`: fresh or missing transcript settings default to `localWhisper / breeze-asr-26`.
- local SQLite setting updated from `parakeet / parakeet-tdt-0.6b-v3-int8` to `localWhisper / breeze-asr-26`.

Validation evidence:

```bash
cargo check --manifest-path frontend/src-tauri/Cargo.toml
git diff --check
```

Results: Rust check passes with existing warnings; diff whitespace check passes.

Dev restart evidence:

```bash
pnpm run tauri:dev
```

Result: Next.js dev server starts on `http://localhost:3118`; the Tauri binary compiles with CUDA enabled and then exits during the Linux single-instance DBus handoff (`org.com_meetily_ai.SingleInstance`). Treat this as a desktop startup layer issue, separate from the ASR default/model-cache change.

Known remaining check:

```bash
pnpm exec tsc --noEmit
```

Result: still blocked by the existing `tests/lib/blocknote-markdown.test.ts` import of `bun:test`; this is not introduced by the Breeze ASR 26 change.

## Validation Evidence

confirmed checks:

```bash
cargo check --manifest-path frontend/src-tauri/Cargo.toml
```

結果：通過；只剩既有 warning。

```bash
curl http://localhost:3118/_next/static/chunks/app/layout.js
curl http://localhost:3118/_next/static/chunks/app/page.js
```

2026-07-08 observation, later superseded by the 2026-07-09 root-layout split:

- `layout.js`: HTTP 200，約 2.79 MB，毫秒級取回。
- `page.js`: HTTP 200，約 1.19 MB，毫秒級取回。
- `.local/tauri-dev.log`: 沒有新的 `ChunkLoadError` 或 CSP refusal；可看到 native API calls。

known remaining check:

```bash
pnpm exec tsc --noEmit
```

結果：仍失敗於既有測試型別問題：

```text
tests/lib/blocknote-markdown.test.ts: Cannot find module 'bun:test'
```

這不是本次改動新增的錯誤，但 commit 前若要整體 typecheck 通過，需要補測試環境型別或調整 tsconfig/test config。

## 2026-07-09 Root Layout Chunk Fix

使用者回報同一類畫面再次出現：

```text
ChunkLoadError: Loading chunk app/layout failed.
timeout: http://localhost:3118/_next/static/chunks/app/layout.js
```

Root cause:

- `frontend/src/app/layout.tsx` was still a client component.
- It imported the app provider stack, Tauri event hooks, toast setup, import dialog logic, sidebar, main content, and onboarding flow from the root layout path.
- That made `app/layout.js` a large dev chunk, about `2.8 MB`, which Tauri WebView could time out while loading.
- When that chunk timed out, HTML/SSR still made the sidebar visible, but React hydration did not finish. The visible UI then had no working event handlers.

Fix:

- `frontend/src/app/layout.tsx` is now a server root layout again.
- The previous client-side shell moved to `frontend/src/app/AppShell.tsx`.
- `frontend/src/app/ClientAppShell.tsx` is a thin client loader that dynamically imports `AppShell` with `ssr: false`.

Validation:

```bash
curl -sS -D - --max-time 10 http://localhost:3118/_next/static/chunks/app/layout.js -o /tmp/meetily-layout-final.js
wc -c /tmp/meetily-layout-final.js
curl -sS -I --max-time 5 http://localhost:3118/
pnpm exec tsc --noEmit
```

Results:

- `app/layout.js`: reduced from about `2.8 MB` to `186,593` bytes.
- `http://localhost:3118/`: HTTP `200`.
- `pnpm run tauri:dev`: restarted successfully; `target/debug/meetily` running.
- Tauri log confirms `provider=localWhisper, model=breeze-asr-26`.
- `pnpm exec tsc --noEmit` still fails only on the existing `tests/lib/blocknote-markdown.test.ts` import of `bun:test`.

Status:

- This is the root-cause fix for the repeated `app/layout.js` chunk timeout.
- It is separate from ASR model runtime quality; Breeze ASR 26 still needs a real recording/transcription check before release-facing ASR quality claims.

## Log And Audit Events

| Event ID | Date | Symptom | Classification | Root cause | Fix / decision | Evidence | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| MEETILY-AUDIT-2026-07-08-001 | 2026-07-08 | Tauri window showed a Next.js overlay: `ChunkLoadError: Loading chunk app/layout failed`; sidebar could appear but buttons were not operable. | Same repeated issue family: Next.js chunk timeout causing incomplete React hydration. | Initial app shell pulled too much client-side UI into startup chunks; Tauri WebView in dev mode timed out while loading `app/layout.js`. | Added dynamic imports for heavy layout/page components, removed the homepage root opacity animation, and relaxed dev-only CSP with `devCsp: null`. | `cargo check --manifest-path frontend/src-tauri/Cargo.toml`; `layout.js` / `page.js` returned HTTP `200`; Tauri log no longer showed new chunk/CSP refusal in that run. | Superseded by MEETILY-AUDIT-2026-07-09-001 because `layout.tsx` still remained a client root and the chunk was still large. |
| MEETILY-AUDIT-2026-07-08-002 | 2026-07-08 | Breeze ASR 25 existed locally but could not be safely selected through `localWhisper`. | Separate issue: model-runtime format mismatch, not the UI hydration failure. | Local Breeze ASR 25 artifact was CTranslate2 / faster-whisper format; `localWhisper` uses whisper-rs / whisper.cpp and expects GGML/GGUF/ggmf. | Kept CT2 Breeze ASR 25 blocked from `localWhisper`; added UI compatibility status so it would not be mistaken for a ready whisper-rs model. | Header inspection showed non-GGML/GGUF bytes; model manager reports `Needs GGML/GGUF`. | Preserved as guardrail. |
| MEETILY-AUDIT-2026-07-08-003 | 2026-07-08 | User requested a Taiwan Mandarin main ASR model through local Whisper. | ASR default activation. | Breeze ASR 26 has a published GGML artifact compatible with localWhisper, while no int8 GGML artifact was available in the checked repository. | Set `localWhisper / breeze-asr-26` as the main ASR model, registered its download URL, downloaded `ggml-breeze-asr-26.bin`, and updated local SQLite `transcript_settings`. | File exists at `~/.local/share/com.meetily.ai/models/ggml-breeze-asr-26.bin`, size `3,094,623,708` bytes; SQLite setting is `localWhisper / breeze-asr-26`; Rust check passes. | Active. Real transcription quality gate still pending. |
| MEETILY-AUDIT-2026-07-09-001 | 2026-07-09 | Same visible UI problem returned: sidebar / blank main area / buttons not usable; screenshot matched prior `ChunkLoadError` behavior. | Same repeated issue family as MEETILY-AUDIT-2026-07-08-001. | Root `frontend/src/app/layout.tsx` was still a client component importing providers, Tauri hooks, toast, import dialog logic, sidebar, main content, and onboarding flow. That kept `app/layout.js` about `2.8 MB`, so Tauri WebView could still hit chunk timeout. | Converted `layout.tsx` back to a server root layout, moved the client provider shell into `AppShell.tsx`, and added thin `ClientAppShell.tsx` dynamic loader with `ssr: false`. | `app/layout.js` reduced to `186,593` bytes; `http://localhost:3118/` returns HTTP `200`; Tauri log confirms `provider=localWhisper, model=breeze-asr-26`; `target/debug/meetily` running. | Root-cause fix for the repeated non-operable UI / `app/layout.js` timeout. |
| MEETILY-AUDIT-2026-07-09-002 | 2026-07-09 | `pnpm exec tsc --noEmit` fails. | Existing test-config issue, not caused by the hydration or ASR changes. | `tests/lib/blocknote-markdown.test.ts` imports `bun:test`, but TypeScript cannot resolve its type declarations in the current check setup. | Left unchanged; documented as a separate blocker before requiring full repo-wide typecheck pass. | Repeated output: `Cannot find module 'bun:test' or its corresponding type declarations.` | Open. |

Audit conclusion:

- The repeated screenshot problem is the same issue family: `app/layout.js` chunk timeout prevents React hydration, so the visible UI has no handlers.
- The 2026-07-08 dynamic-import work reduced some pressure but did not fully remove the root cause because `layout.tsx` itself stayed client-side.
- The 2026-07-09 root-layout split is the direct root-cause fix: keep root `layout.tsx` server-side and keep heavy providers / Tauri event logic out of the root layout chunk.
- ASR model work is separate: Breeze ASR 26 is active as the localWhisper default, but recording quality still requires a real transcription run.

## First Launch Cost And Faster Next Launches

confirmed first-launch costs:

- Next.js 第一次建立 `.next` cache。
- Rust / Tauri 第一次建立 `target` cache。
- onboarding 觸發模型下載：
  - Parakeet model 約 639 MB。
  - Summary model 約 2.6 GB。

confirmed local cache:

```text
/home/jnclaw/.local/share/com.meetily.ai/models
```

目前約 3.2 GB。保留 `.next/`、`target/`、models cache，下一次啟動會少掉大部分首次成本。

After adopting Breeze ASR 26, the local model cache is about 6.1 GB and includes:

- `ggml-breeze-asr-26.bin`: 3,094,623,708 bytes.
- Parakeet model directory.
- Summary model directory.

## Connection Map

- [`2026-07-14 Linux PipeWire build activation gate`](../audit-events/2026-07-14-linux-pipewire-build-activation-gate/audit-event.md): current-host follow-up；CUDA detection 與 Next.js ready 後，Rust build 在 `libpipewire-0.3-dev` activation gate 停止。這份本機 prerequisite evidence 與本節保存的前次 successful startup evidence 共同建立啟動狀態時間線。
- `docs/architecture.md`: Meetily 的 Tauri / Next.js / Rust core 架構入口；本紀錄補充「SSR、hydration、Tauri command」在實務除錯中的關係。
- `docs/BUILDING.md`: build 與 GPU 自動偵測入口；本紀錄補充 dev mode 首次編譯、cache、模型下載成本。
- `docs/GPU_ACCELERATION.md`: whisper-rs GPU backend 說明；本紀錄補充模型格式 gate，不把 CT2 model 當 whisper-rs model。
- `frontend/README.md`: supported app run path；本紀錄確認目前正確路徑是 `frontend/` 下的 `pnpm run tauri:dev`。
- `CLAUDE.md`: repo-local agent / architecture context；本紀錄沿用「Tauri desktop app 是 supported path，legacy backend 不作為當前執行路徑」。
- `frontend/src-tauri/src/whisper_engine/whisper_engine.rs`: whisper-rs runtime、模型狀態、模型載入驗證的核心位置。
- `frontend/src-tauri/src/config.rs`: localWhisper default model and ASR catalog.
- `frontend/src/constants/modelDefaults.ts`: frontend default ASR model mapping.
- `frontend/src-tauri/src/whisper_engine/commands.rs`: Tauri command 層與 standalone model discovery。
- `frontend/src/components/WhisperModelManager.tsx`: 使用者看得到的模型狀態與可操作按鈕。
- `frontend/src/app/layout.tsx` and `frontend/src/app/page.tsx`: Next.js App Router 初始載入與 hydration 風險面。

## Important Knowledge Not Yet Discussed Enough

### 1. Hydration failure should be debugged as a pipeline

Use this order:

1. HTML response exists.
2. JS chunks return HTTP 200 quickly.
3. CSP does not block scripts.
4. React hydration runs.
5. Tauri bridge calls arrive in Rust logs.
6. UI event handlers trigger commands/events.

Skipping directly to UI code often wastes time.

### 2. Model compatibility should be explicit in the product

For local AI apps, model state should distinguish:

- missing
- downloading
- downloaded but corrupted
- downloaded but incompatible runtime
- ready

The Breeze change adds an incompatible-runtime state using the existing `Error` status. A future cleaner version could add a dedicated `Incompatible` variant, but that is only worth doing when multiple incompatible model families are supported.

### 3. Dev fixes must not silently weaken production security

`devCsp: null` is acceptable for development triage. It should not become a reason to relax production `csp`. Production security should keep explicit `script-src`, `connect-src`, and asset boundaries.

### 4. Cache is part of developer experience

Fast second launch depends on preserving:

- `frontend/.next/`
- Rust `target/`
- `~/.local/share/com.meetily.ai/models/`
- pnpm store

Deleting all caches is useful only when diagnosing stale build artifacts.

### 5. Commit hygiene matters

Before commit:

- separate functional changes from formatter-only changes;
- keep Breeze compatibility work separate from Next.js hydration work if possible;
- keep documentation in a third commit if publishing separately;
- push to the fork remote, not upstream main.

## Course Map For CS Students

| Topic | What this case teaches | Relevant CS course |
| --- | --- | --- |
| SSR and hydration | HTML can render while buttons remain dead | Web Programming, Frontend Engineering |
| Code splitting | Smaller initial chunks reduce startup failure risk | Web Programming, Software Performance |
| CSP | Dev security policy can block app runtime | Web Security, Network Security |
| Tauri bridge | Frontend events call Rust commands through IPC | Systems Programming, Software Architecture |
| Rust model management | enum states, async file checks, runtime validation | Rust / Systems Programming |
| Local ASR runtime | whisper-rs vs faster-whisper model formats | Machine Learning Systems, Speech Processing |
| GPU acceleration | CUDA feature flags, runtime backend selection | Parallel Programming, GPU Computing |
| Observability | Logs, curl checks, process checks, cache inspection | Software Engineering, DevOps |
| Product error states | Disable impossible actions; show actionable reason | Human-Computer Interaction |
| Version control | Small commits, diff hygiene, fork remote discipline | Software Engineering Project |

## Action Register

| ID | Question / action | Owner | Due / trigger | Evidence needed |
| --- | --- | --- | --- | --- |
| A1 | Validate Breeze ASR 26 runtime quality in Meetily with one real localWhisper transcription | Jason / next agent | Before release-facing ASR quality claim | Successful model load, real transcription, transcript output reviewed |
| A2 | Keep Meetily code, docs, and planning mirror as separate commits | next agent | During publish | Implementation commit `a85d7ceea8e09eb9ed5796ef9d4f9f8053aab6d3`; separate documentation and planning commit hashes |
| A3 | Fix `bun:test` TypeScript test config | next agent | Before requiring full `pnpm exec tsc --noEmit` pass | Typecheck passes or test files excluded through intended config |
| A4 | Consider production build path for faster normal launch | Jason / next agent | After dev flow is stable | `pnpm run tauri:build` or selected release binary starts without dev compile |
| A5 | Keep push target on fork remote | next agent | Before publish | remote points to `https://github.com/JasonLn0711/meetily.git` or equivalent fork remote |
| A6 | Do not set CT2 Breeze ASR 25 as `localWhisper` main model | next agent | Until compatible artifact/provider exists | GGML/GGUF artifact or CTranslate2 provider implementation with a real transcription |
| A7 | Keep Breeze ASR 26 local model cache available between launches | next agent | Before diagnosing slow first launch again | `~/.local/share/com.meetily.ai/models/ggml-breeze-asr-26.bin` exists and matches expected size |
| A8 | Keep root `layout.tsx` server-side and avoid reintroducing heavy provider imports there | next agent | Any future shell/provider edit | `app/layout.js` remains small enough for Tauri WebView dev loading |

## Current Status

- source preserved: yes, this note records the user-visible errors, model-format finding, changed files, and validation evidence.
- adopted decision: Next/Tauri dev hydration fix, Breeze CT2 compatibility gate, Breeze ASR 26 localWhisper default, and the 2026-07-09 root-layout chunk split are adopted in current working tree.
- validated: Rust check passes; `app/layout.js` reduced to `186,593` bytes; app route returns HTTP `200`; Breeze ASR 26 file exists with expected size.
- startup status: `pnpm run tauri:dev` rebuilds with CUDA and `target/debug/meetily` is running.
- implementation pending: Breeze ASR 26 still needs one real transcription run before release-facing quality claims; CT2 Breeze ASR 25 remains intentionally blocked from `localWhisper`.
- publication path: implementation, engineering documentation, and planning mirror are handled as separate publish units; push target is Jason's fork, not the upstream repository.

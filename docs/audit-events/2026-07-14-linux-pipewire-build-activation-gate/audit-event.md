# Meetily Linux PipeWire Build Activation Gate Audit Event

## 事件識別

- Event ID：`AUDIT-2026-07-14-MEETILY-LINUX-PIPEWIRE-BUILD-003`
- 事件日期：`2026-07-14`（Asia/Taipei；原始 terminal log 未提供 wall-clock timestamp）
- Audit record time：`2026-07-14 07:59:32 CST`
- Implementation update：`2026-07-14 08:08:58 CST`
- 事件類型：Linux local-development prerequisite activation gate
- 目前狀態：`source preserved`、`analysis validated`、`preflight implementation validated`、`workstation package activation pending`
- Canonical home：Meetily execution repo
- Related hardening event：[`AUDIT-2026-07-14-MEETILY-AUDIO-GPU-HARDENING-002`](../2026-07-14-audio-owner-gpu-asr-hardening/audit-event.md)

## FIRST PRINCIPLE routing

```text
scarce_resource: reproducible local-build evidence and maintainer attention
canonical_home: Meetily docs/audit-events
planning_role: existing planning status remains stable; this event updates local activation evidence only
evidence_path: source.log + pkg-config/dpkg diagnostics + BUILDING.md + Linux workflow dependency declarations
next_gate: install the documented PipeWire and PulseAudio development packages, rerun pnpm run tauri:dev, and preserve the result
```

## 結論

本次 `pnpm run tauri:dev` 已完成 supply-chain lockfile policy、CUDA backend 自動偵測與 Next.js dev server 啟動，Rust/Tauri build 隨後在 `libspa-sys v0.10.0` 的 custom build command 停止。直接原因是本機缺少提供 `libpipewire-0.3.pc` 的 `libpipewire-0.3-dev` package；`pkg-config` 因此無法解析 `libpipewire-0.3 >= 0.3`，Cargo 以 exit status `101` 結束。

這是一個已確認的本機 toolchain activation gate。GPU detector 已選定 CUDA；本輪證據範圍到 Rust dependency compilation 為止，ASR inference、audio capture 與桌面應用啟動由 package activation 後的 rerun 接續驗證。Repo 已在 `docs/BUILDING.md` 與 Linux workflows 宣告 PipeWire/PulseAudio development packages；目前工作站的下一動作是完成同一 prerequisite activation。

## Source preservation

完整 user-provided terminal output 原文保存在 [`source.log`](source.log)。來源層維持原始文字，audit interpretation 獨立收錄於本文。

```text
bytes: 5221
lines: 122
sha256: deb0e50d3ef5c015398a0ab4b6bd6f1aae18aaabfa955a83ef3d0e02a21d6333
```

來源包含：

- 原始 shell prompt、啟動命令與 dependency-install output；
- CUDA auto-detection 與 Tauri feature selection；
- Next.js local/network endpoints 與 ready timing；
- Cargo compile sequence；
- `libspa-sys` panic、完整 `pkg-config` environment probes、缺少 `.pc` file 訊息；
- final `ELIFECYCLE` exit status `101`。

## 事件時間線

原始 log 未含 wall-clock timestamps；下列順序完全依 terminal event order。

| 順序 | Evidence label | Event |
|---:|---|---|
| 1 | `confirmed` | `pnpm run tauri:dev` 執行；lockfile policy 通過且 dependencies 已是最新狀態。 |
| 2 | `confirmed` | `scripts/tauri-auto.js` 偵測 NVIDIA GPU 與 CUDA，設定 Linux/CUDA CMake flags。 |
| 3 | `confirmed` | Tauri 執行 `pnpm dev`；Next.js 15.5.20 在 port `3118` 於 `1469ms` ready。 |
| 4 | `confirmed` | Cargo 使用 `cuda` 與 `platform-default` features 編譯 Tauri core。 |
| 5 | `confirmed` | `libspa-sys v0.10.0` build script 呼叫 `pkg-config` 查詢 `libpipewire-0.3 >= 0.3`。 |
| 6 | `confirmed` | `libpipewire-0.3.pc` 無法被找到；custom build command 以 `101` 結束。 |
| 7 | `confirmed` | pnpm 回報兩層 `ELIFECYCLE` failure；本輪未進入桌面 runtime。 |

## Direct cause and scope

### Direct cause — confirmed

`libspa-sys` 是 PipeWire Rust binding 的 system-library layer。它的 build script 要求 `pkg-config` 找到 `libpipewire-0.3` metadata；本機未安裝 `libpipewire-0.3-dev`，也沒有 `/usr/lib/x86_64-linux-gnu/pkgconfig/libpipewire-0.3.pc`，所以 build 在 native dependency discovery 階段停止。

### Current workstation diagnostics — validated

`2026-07-14 07:59:32 CST` 重新檢查目前主機：

```text
pkg-config --modversion libpipewire-0.3: exit 1, package not found
libpipewire-0.3-dev: not installed
libpulse-dev: not installed
libasound2-dev: install ok installed, version 1.2.11-1ubuntu0.2
/usr/lib/x86_64-linux-gnu/pkgconfig/libpipewire-0.3.pc: absent
```

`libpulse-dev` 是既有 Linux build contract 的另一個 audio prerequisite。本 log 尚未執行到可證明它會成為下一個 compiler failure 的階段；它仍應與 PipeWire package 一起完成 activation。

### Capability already demonstrated — confirmed

- Node/pnpm dependency preparation可執行，supply-chain lockfile policy 通過。
- GPU detector 可辨識 NVIDIA CUDA toolchain。
- Next.js development server 可啟動並進入 ready。
- Cargo 已開始 CUDA Tauri dependency graph compilation。

### Activation scope — pending validation

本事件保留 build preflight 證據。Remediation 後的 rerun 將驗證 Meetily desktop process、Tauri window、audio device access、Breeze ASR 26 model load 與真實 CUDA inference。

## Adopted remediation path

Ubuntu/Debian 的最小一致修復沿用 canonical build contract：

```bash
sudo apt install libpipewire-0.3-dev libpulse-dev
pkg-config --modversion libpipewire-0.3
cd frontend
pnpm run tauri:dev
```

目前 `libasound2-dev` 已安裝，remediation 集中於 PipeWire 與 PulseAudio packages。當 package 安裝在非標準 prefix 時，以實際 `libpipewire-0.3.pc` parent directory 設定 `PKG_CONFIG_PATH`；本機診斷對應的啟動路徑是安裝 development package。

## Fail-fast implementation

[`frontend/scripts/tauri-auto.js`](../../../frontend/scripts/tauri-auto.js) 現在於 Cargo 啟動前以 `pkg-config` 檢查 `alsa`、`libpipewire-0.3` 與 `libpulse`。檢查會將缺少的 metadata 映射為 Ubuntu/Debian development package，並回報可直接執行的 `sudo apt install ...` activation command。

[`frontend/package.json`](../../../frontend/package.json) 已將自動偵測與顯式 CUDA、Vulkan、Metal、CoreML、HIP dev/build scripts 全部路由至同一 wrapper。這個共用 preflight 保護 supported Tauri entry points，並保留 `TAURI_GPU_FEATURE` environment override。

Validation：

```text
pnpm test: 11 passed, 0 failed
tauri-auto prerequisite tests: 2 passed, 0 failed
targeted ESLint: passed
missing-host check: exit 1 with libpipewire-0.3-dev and libpulse-dev activation command
```

Regression check 保存在 [`frontend/tests/lib/tauri-auto.test.mjs`](../../../frontend/tests/lib/tauri-auto.test.mjs)，涵蓋 all-ready 與 two-package activation 兩條路徑。

Commit evidence：

- `83b4b52` — preserve source log, audit interpretation, index, and documentation connections.
- `bfe643b` — route supported Tauri scripts through the shared Linux audio prerequisite preflight and add regression checks.

## Workstation activation record

`2026-07-14` 的 package installation 已進入系統授權 gate：

```text
command: sudo apt-get install -y libpipewire-0.3-dev libpulse-dev
result: sudo requires interactive terminal authentication
```

Desktop PolicyKit 路徑可開啟系統認證；本輪在認證尚未完成時受控停止，不接觸或保存使用者密碼。主機 activation 的最短路徑是由 workstation owner 在自己的 terminal 執行上述指令；之後的 `pnpm run tauri:dev` 將由新 preflight 先確認 prerequisites，再進入 CUDA/Tauri build。

## Connection map

| 入口 | 連結目的 |
|---|---|
| [`docs/audit-events/README.md`](../README.md) | Meetily audit events 的最小索引與主要 retrieval path。 |
| [`Audio owner thread / GPU-only ASR hardening audit`](../2026-07-14-audio-owner-gpu-asr-hardening/audit-event.md) | 說明 PipeWire dependencies 進入 Linux workflows 的 implementation provenance；本事件記錄本機 activation gate。 |
| [`docs/BUILDING.md`](../../BUILDING.md) | Ubuntu/Debian prerequisites、run command 與本事件的 recovery/validation 入口。 |
| [`docs/GPU_ACCELERATION.md`](../../GPU_ACCELERATION.md) | CUDA backend detection 與 hardware-backed inference evidence gate；本事件只到 build preflight。 |
| [`.github/workflows/build-linux.yml`](../../../.github/workflows/build-linux.yml) | Supported Ubuntu build image 的 canonical package installation list。 |
| [`frontend/scripts/tauri-auto.js`](../../../frontend/scripts/tauri-auto.js) | Linux audio prerequisite fail-fast implementation 與 shared GPU entry point。 |
| [`frontend/tests/lib/tauri-auto.test.mjs`](../../../frontend/tests/lib/tauri-auto.test.mjs) | all-ready 與 missing-package regression evidence。 |
| [`frontend/src-tauri/Cargo.toml`](../../../frontend/src-tauri/Cargo.toml) | `cpal` 的 Linux `pipewire` / `pulseaudio` feature declaration。 |
| [`2026-07-08 Tauri startup engineering note`](../../engineering-notes/2026-07-08-tauri-next-hydration-breeze-asr.md) | 前次可啟動狀態與 startup pipeline；本事件是獨立的 current-host prerequisite gate。 |

## Unresolved action ledger

| ID | Question / action | Owner | Due / trigger | Evidence needed |
|---|---|---|---|---|
| `BUILD-DONE-001` | Supported Tauri scripts 共用 Linux audio fail-fast preflight | Meetily build owner | completed 2026-07-14 | `11` Node tests、targeted ESLint、actionable missing-host output |
| `BUILD-NEXT-001` | 安裝 `libpipewire-0.3-dev` 與 `libpulse-dev` | Meetily workstation owner | 下一次 local dev run 前 | `dpkg-query` status、`pkg-config --modversion libpipewire-0.3` |
| `BUILD-NEXT-002` | 重跑 `pnpm run tauri:dev` | Meetily workstation owner | packages activated 後 | exit status、Tauri process/window ready markers、完整 failure log（若仍失敗） |
| `BUILD-NEXT-003` | 驗證 audio/runtime path | Meetily runtime owner | desktop app ready 後 | device enumeration、controlled start/stop、runtime log |

## Scope controls

- 本 audit 完成原始來源保存、direct-cause validation、canonical remediation adoption 與相關文件連結。
- 本次執行範圍已完成 system diagnostics、durable documentation 與 fail-fast application code；package activation 與 desktop runtime 保持為后續獨立驗證層。
- CUDA auto-detection success 是 build configuration evidence；真實 ASR inference 仍沿用 GPU audit 的 live-evidence contract。
- CI/workflow dependency declaration保持有效；本事件新增的是目前 workstation 的 activation status，不改寫已發布 implementation history。

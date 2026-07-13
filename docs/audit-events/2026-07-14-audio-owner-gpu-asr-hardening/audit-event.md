# Meetily Audio Owner Thread 與 GPU-only ASR Hardening Audit Event

## 事件識別

- Event ID：`AUDIT-2026-07-14-MEETILY-AUDIO-GPU-HARDENING-002`
- 事件時間：`2026-07-13 22:55:44` 至 `2026-07-14 00:23:33`（Asia/Taipei；以 implementation commit 時間為準）
- Audit record closeout：`2026-07-14 00:49 CST`（Asia/Taipei；文件、連結與目前工作樹重新驗證）
- 事件類型：native audio lifecycle remediation、dead-path deletion、model capability routing、GPU-only ASR 與 release activation
- 目前狀態：`source preserved`、`adopted decision`、`implementation validated`、`implementation published`
- Canonical home：Meetily execution repo
- 前置事件：[`AUDIT-2026-07-13-MEETILY-UNCONTROLLED-SHUTDOWN-001`](../2026-07-13-uncontrolled-shutdown/audit-event.md)
- Cross-repo evidence owner：[`JasonLn0711/project_aura`](https://github.com/JasonLn0711/project_aura)

## FIRST PRINCIPLE routing

```text
scarce_resource: native audio reliability、transcription trust、GPU capacity、maintainer attention
canonical_home: Meetily owns product runtime; Project AURA owns paired evaluation artifacts
planning_role: locator、status、capacity、publish evidence、next gate
evidence_path: this audit + prior source.log + AURA paired live artifact set
next_gate: long-duration mic+system audio / hot-plug / controlled shutdown and broader ASR corpus
```

## 結論

本事件關閉兩個最高價值的架構風險：

1. `cpal::Stream` 的建立、播放、重連、停止與 drop 現在全部留在 dedicated owner thread；UI／Tauri command 只透過 channel 傳遞 start、stop 與 shutdown 意圖。先前四層 `unsafe impl Send` 已移除。
2. ASR 現在只在 activated GPU backend 上執行。Whisper 在 model context 建立前驗證 compiled backend 與 runtime GPU；Parakeet 明確使用 CUDA Execution Provider，並停用 ONNX Runtime CPU provider fallback。

同時完成 2,242 行未接入產品路徑的舊音訊監控／post-processing／UI surface 刪除、Parakeet language capability routing、Breeze ASR 26 onboarding 對齊、Linux PipeWire build dependencies、release benchmark adapter、GPU-enabled workflows 與可重現 CUDA release flags。

Native audio ownership 已由 25 次真實 microphone lifecycle 驗證；Meetily Breeze ASR 26 已在 NVIDIA CUDA release runtime 完成 10 次真實臺灣華語 inference。原始 heap-corruption 事件的 unsafe ownership path 已消除；長時間雙音源、hot-plug 與受控 shutdown 壓力測試仍是下一可靠性 gate。

## 與前置 crash audit 的關係

### 原始 direct cause — confirmed and preserved

2026-07-13 原始事件中，glibc 偵測 `corrupted double-linked list` 並 abort process。完整 73,837-byte terminal log、SHA-256、時間線與 root-cause analysis 持續保存在前置 audit；本事件不改寫該原始記錄。

### Scope change：最高風險 ownership path 已移除

前置 audit 找到四層 `unsafe impl Send`，讓含 `cpal::Stream` 的物件可跨 thread 移動。Commit `1f4be5d` 以 owner-thread architecture 取代這條路徑，並將 CPAL 升至 `0.18.1`。因此「unsafe ownership/race cause」從 active implementation risk 轉為 removed path；這是設計控制的實作完成，不等同於已由 core dump 反證原始 first invalid write。

### Scope change：高頻 native enumeration 已收斂

Commit `44c468e` 讓 Linux device discovery 每個方向只列舉一次，停止額外 `host.devices()` 全掃描；stable device interval 實際使用 5 秒，device missing 時使用 2 秒。這完成前置 audit 的最小 containment。

### Pending validation：長時間與裝置變更

25 次 default-microphone lifecycle 驗證 owner thread 的 repeated start／callback／stop／drop。完整 acceptance 仍需要長時間 microphone + system audio、default-device change、hot-plug、受控 shutdown markers 與 artifact integrity。若 allocator abort 再現，core dump／sanitizer 應保存 first invalid write/free stack。

## Source preservation

| Source | Canonical location | Evidence role |
|---|---|---|
| 原始 uncontrolled-shutdown terminal log | [`../2026-07-13-uncontrolled-shutdown/source.log`](../2026-07-13-uncontrolled-shutdown/source.log) | crash direct cause、最後事件、正常 shutdown markers 缺席 |
| 原始 crash interpretation | [`../2026-07-13-uncontrolled-shutdown/audit-event.md`](../2026-07-13-uncontrolled-shutdown/audit-event.md) | root-cause hypotheses、remediation order、acceptance criteria |
| Native audio implementation | commits `44c468e`, `9434e9f`, `1f4be5d` | containment、deletion、owner thread、CPAL upgrade |
| ASR implementation | commits `528cad1..69e3e3c` | language、model、CI、benchmark、GPU-only、release activation |
| Live ASR artifacts | [AURA benchmark artifact set](https://github.com/JasonLn0711/project_aura/tree/main/artifacts/asr-benchmark/2026-07-13-common-voice24-minimum) | real CUDA inference、transcripts、timestamps、GPU telemetry、errors |

## 事件時間線

| 時間（Asia/Taipei） | Commit | Event |
|---|---|---|
| Earlier containment | `44c468e` | Linux device discovery 收斂為 input/output 各一次；stable/missing monitor interval 具可測試 policy。 |
| 2026-07-13 22:55:44 | `9434e9f` | 刪除 2,242 行 inactive monitoring、system detector、post-processor 與重複 UI。 |
| 2026-07-13 23:07:36 | `1f4be5d` | 建立 `AudioRuntime` owner thread、移除 unsafe Send、升級 CPAL 0.18.1。 |
| 2026-07-13 23:18:02 | `528cad1` | Parakeet v2/v3 language capability contract；zh-TW 進入 Local Whisper。 |
| 2026-07-13 23:22:38 | `0259461` | onboarding download 使用 configured Breeze ASR 26 model。 |
| 2026-07-13 23:23:17 | `7a39693` | Linux workflows 安裝 CPAL PipeWire／PulseAudio development packages。 |
| 2026-07-13 23:29:12 | `bff4cb9` | 新增 release-facing `asr_benchmark` example。 |
| 2026-07-14 00:03:05 | `4a97604` | Whisper／Parakeet GPU-only runtime、auto-detection、scripts、Cargo features 與 docs 收斂。 |
| 2026-07-14 00:03:13 | `182b551` | Linux／Windows CI 改用 GPU-enabled Vulkan release path。 |
| 2026-07-14 00:23:33 | `69e3e3c` | CUDA compute matrix、PIC flags 與 `nvidia-smi` runtime detection 可重現。 |

## Native audio implementation record

### Device enumeration containment

[`frontend/src-tauri/src/audio/devices/platform/linux.rs`](../../../frontend/src-tauri/src/audio/devices/platform/linux.rs) 使用同一 host 各列舉一次 input 與 output，並從 input monitor source 建立 system-audio candidate。Linux 路徑不再回到 `host.devices()` 做第三次全裝置掃描。

[`frontend/src-tauri/src/audio/device_monitor.rs`](../../../frontend/src-tauri/src/audio/device_monitor.rs) 的 `monitor_check_interval()` 讓 stable device 使用 5 秒 interval、missing device 使用 2 秒；loop 會把 `next_interval` 寫回 active interval。Regression test 保護兩種狀態。

### Inactive surface deletion

Commit `9434e9f` 刪除 inactive async logger、batch processor、monitoring、post-processing、system detector、舊 TypeScript audio types 與未使用 UI components，淨減少 2,242 行。Native lifecycle analysis 現在只需追蹤 active capture／pipeline／owner runtime。

### Dedicated owner thread

[`frontend/src-tauri/src/audio/audio_runtime.rs`](../../../frontend/src-tauri/src/audio/audio_runtime.rs) 建立 named OS thread `meetily-audio-owner`，thread 內使用 current-thread Tokio runtime 與 `LocalSet`。`AudioStreamManager` 是 owner loop 的 local state，不離開 thread。

Command contract：

- `Start`：傳遞 recording state、microphone、system device 與 oneshot reply。
- `Stop`：在 owner thread 內停止並 drop manager，將 active count 歸零。
- `Shutdown`：結束 loop；Drop 送出 shutdown 並 join owner thread。
- Channel capacity：8；command failure 與 owner termination 都回傳具體 error。

### Unsafe ownership bypass removal

下列 declarations 已移除：

- `unsafe impl Send for StreamBackend`
- `unsafe impl Send for AudioStream`
- `unsafe impl Send for AudioStreamManager`
- `unsafe impl Send for RecordingManager`

[`frontend/src-tauri/src/audio/recording_manager.rs`](../../../frontend/src-tauri/src/audio/recording_manager.rs) 現在持有 `AudioRuntime` handle；start、stop、cleanup 與 reconnect 均透過 owner runtime。`AudioStreamManager` 的 visibility 收斂為 `pub(crate)`。

### CPAL release line

Linux、macOS 與 Windows target dependencies 均 pin `cpal = "=0.18.1"`。這個版本高於前置 audit 使用的 `0.15.3`，並涵蓋前置 audit 引用的 ALSA stream-shutdown race 修復版本線。

## ASR implementation record

### Whisper GPU contract

[`frontend/src-tauri/src/whisper_engine/acceleration.rs`](../../../frontend/src-tauri/src/whisper_engine/acceleration.rs) 以 `WhisperCompiledBackend` 表達 Metal、CUDA、Vulkan、HIP BLAS 與 diagnostic-only CPU state。`whisper_context_acceleration_for()` 要求 runtime detection 與 compiled backend 相符；CPU compiled state 在所有 runtime detections 下都回傳 activation error。成功路徑固定 `use_gpu=true` 與 `gpu_device=0`；flash attention 只在 Metal／CUDA High 或 Ultra tiers 啟用。

`WhisperEngine` 只在 acceleration validation 成功後建立 model context。

### Parakeet CUDA contract

[`frontend/src-tauri/src/parakeet_engine/model.rs`](../../../frontend/src-tauri/src/parakeet_engine/model.rs) 的 session builder：

- 只註冊 `CUDAExecutionProvider`；
- 使用 `error_on_failure()`；
- 設定 `session.disable_cpu_ep_fallback=1`；
- 未編譯 `cuda` feature 時回傳 `GpuBackendRequired`。

這使 Parakeet 具單一可稽核 execution-provider path。

### Model-language routing

[`frontend/src-tauri/src/parakeet_engine/capabilities.rs`](../../../frontend/src-tauri/src/parakeet_engine/capabilities.rs) 於模型載入／推論前驗證 request：v3 支援 auto 與 25 種正式歐洲語言；v2 為 English-only；`auto-translate` 與 zh-TW 路由至 Local Whisper。這份 contract 同時套用 live transcription、import 與 retranscription paths。

### Onboarding model alignment

Commit `0259461` 讓初次下載、重試與 background download 都使用 `DEFAULT_WHISPER_MODEL`，對齊 Breeze ASR 26 localWhisper default，讓 onboarding 與 runtime 使用同一 model identity。

### GPU build and release activation

- CPU／OpenBLAS ASR feature 與 npm scripts 已移出 supported surface。
- auto-detection 找不到可用 GPU toolchain 時回傳 nonzero activation error。
- Linux／Windows workflows 使用 Vulkan GPU build；macOS 使用 Metal。
- NVIDIA CUDA helper 預設編譯 architectures `75;86;89`。
- CUDA builds 設定 `CMAKE_POSITION_INDEPENDENT_CODE=ON` 與 `CMAKE_CUDA_FLAGS=-Xcompiler=-fPIC`。
- `nvidia-smi` 必須成功且回傳非空輸出，才標記 CUDA runtime available。

## Validation record

### Native audio owner lifecycle

```text
audio::audio_runtime::tests::live_default_microphone_survives_repeated_owner_lifecycles
25 cycles: start -> first callback -> stop -> active_stream_count == 0
result: passed
```

每個 cycle 要求 3 秒內收到真實 microphone callback；test 會在 callback 缺席、channel 提前關閉、start/stop failure 或 active count 不一致時失敗。

### GPU policy tests

```text
whisper_engine::acceleration::*
5 passed; 0 failed

parakeet_cuda_provider_is_an_explicit_activation_gate
1 passed; 0 failed

parakeet_engine::capabilities::*
4 passed; 0 failed
```

測試涵蓋 CUDA success、CUDA/runtime mismatch、Vulkan GPU requirement、flash-attention scope、所有 CPU compiled-state rejection，以及 v2／v3 language capability、translation scope 與 zh-TW routing。

### Release CUDA build

`asr_benchmark` 使用 release profile 與 `--features cuda` 完成 fresh target build。第一次真實 release link 暴露 CUDA non-PIC relocation；commit `69e3e3c` 修正後 release build 通過。這把「Cargo check 可用」提升為「實際 release binary 可連結與執行」。

### Live ASR inference

同一組 5 個 CC0 Common Voice 24 zh-TW WAV 各重複 2 次：

| Runtime | Validity | Runs | Exact | Mean CER | Mean runtime | Mean RTF | Model load | Max GPU utilization |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| Meetily Breeze ASR 26 / whisper-rs | `valid_target_runtime` | 10 | 8 | 0.0571 | 0.196 s | 0.076 | 0.729 s | 89% |

Runtime stderr 保存 `use gpu = 1`、`flash attn = 1`、RTX 4090 Laptop GPU、compute capability 8.9 與 CUDA model allocation 3093.99 MB。

Canonical live evidence：[`project_aura/artifacts/asr-benchmark/2026-07-13-common-voice24-minimum/`](https://github.com/JasonLn0711/project_aura/tree/main/artifacts/asr-benchmark/2026-07-13-common-voice24-minimum)。

### Audit record integrity

- `2026-07-14` 重新執行 25-cycle default-microphone owner lifecycle test：通過。
- 重新執行 Whisper GPU policy、Parakeet CUDA activation 與 model-language capability tests：共 `10` tests passed。
- AURA 與 Meetily 本次 audit／backlink 文件共 `75` 個本機相對連結完成解析。
- AURA、Meetily、planning 三個 repo 的 `git diff --check` 均通過；planning knowledge validator 通過 `157` notes／catalog entries。

## Previous audit recommendation closeout

| Previous recommendation | Evidence label | Current disposition |
|---|---|---|
| 收斂錄音期間完整 ALSA scans | `validated` | `44c468e`：Linux input/output 各一次；stable interval 5 秒。 |
| CPAL stream 移至 owner thread | `validated` | `1f4be5d`：dedicated thread + channel；四層 unsafe Send 移除。 |
| 升級 CPAL 並驗證 shutdown race fix line | `validated implementation` | target dependencies pin 0.18.1；25-cycle mic lifecycle passed。 |
| 啟用 core dump／sanitizer | `deferred activation` | 原始 event 未再現；若再現立即保存 first invalid native stack。 |
| 長時間 mic + system／hot-plug／shutdown markers | `pending confirmation` | owner architecture 已就緒；完整壓力 evidence 尚待執行。 |

## Decision register

| Decision | Evidence label | Adopted action |
|---|---|---|
| Native stream ownership | `confirmed` | stream lifetime 只由 `meetily-audio-owner` thread 擁有。 |
| Audio control interface | `confirmed` | callers 傳送 intent；native stream 不跨 command/thread boundary。 |
| Dead monitoring surfaces | `confirmed` | active module tree 未使用者已刪除；Git history 保留 provenance。 |
| Whisper ASR runtime | `confirmed` | only activated Metal／CUDA／Vulkan／HIP backend。 |
| Parakeet ASR runtime | `confirmed implementation` | CUDA EP only；CPU EP fallback disabled。 |
| Parakeet zh-TW | `deferred activation` | Local Whisper 擁有臺灣華語路徑。 |
| Product default | `confirmed` | Breeze ASR 26 through Local Whisper。 |
| Original heap first invalid write | `pending confirmation` | core／sanitizer evidence 僅在 recurrence 或 focused reproduction 啟動。 |

## Connection map

| 入口 | 連結目的 |
|---|---|
| [`docs/audit-events/README.md`](../README.md) | 所有 Meetily audit events 的最小索引。 |
| [`2026-07-13 uncontrolled shutdown`](../2026-07-13-uncontrolled-shutdown/audit-event.md) | 原始 log、direct cause、pre-fix analysis 與原始 acceptance criteria。 |
| [`docs/architecture.md`](../../architecture.md) | 產品 component ownership 與 engineering note 入口。 |
| [`docs/GPU_ACCELERATION.md`](../../GPU_ACCELERATION.md) | GPU backend activation、readiness 與 evidence gate。 |
| [`docs/BUILDING.md`](../../BUILDING.md) | release toolchain、runtime contract、validation command 與 CUDA PIC controls。 |
| [`audio_runtime.rs`](../../../frontend/src-tauri/src/audio/audio_runtime.rs) | CPAL owner-thread canonical implementation。 |
| [`acceleration.rs`](../../../frontend/src-tauri/src/whisper_engine/acceleration.rs) | Whisper GPU invariant canonical implementation。 |
| [`parakeet model.rs`](../../../frontend/src-tauri/src/parakeet_engine/model.rs) | Parakeet CUDA-only session canonical implementation。 |
| [`parakeet capabilities.rs`](../../../frontend/src-tauri/src/parakeet_engine/capabilities.rs) | model-language capability canonical contract。 |
| [`asr_benchmark.rs`](../../../frontend/src-tauri/examples/asr_benchmark.rs) | reference-backed real inference adapter。 |
| [AURA counterpart audit](https://github.com/JasonLn0711/project_aura/blob/main/docs/audit-events/2026-07-14-gpu-only-asr-live-benchmark/audit-event.md) | paired protocol、AURA CUDA policy、20-run artifacts 與 cross-repo decision。 |
| `planning-everything-track/data/projects/2026-05-project-aura-refactor.md` | status、capacity、publish evidence、next gate only。 |

## Unresolved question and action ledger

| ID | Question / action | Owner | Due / trigger | Evidence needed |
|---|---|---|---|---|
| `AUDIO-NEXT-001` | 長時間 microphone + system audio soak | Meetily runtime owner | 下一次 dedicated reliability block | duration、shutdown markers、artifact integrity、exit status |
| `AUDIO-NEXT-002` | hot-plug／default-device change／reconnect | Meetily runtime owner | soak run 同批 | device events、owner commands、callback continuity、no abort |
| `AUDIO-NEXT-003` | 原始 crash focused reproduction | Meetily runtime owner | allocator abort recurrence 或 soak failure | core dump／ASan／Valgrind first invalid stack |
| `ASR-NEXT-001` | 長音訊、遠距、重疊與雜訊 benchmark | AURA + Meetily evaluation owners | licensed reference set ready | CER、correction time、VRAM、cancel/recovery logs |
| `ASR-NEXT-002` | Parakeet CUDA real inference | Meetily ASR owner | supported-language corpus ready | CUDA EP proof、audio、transcript、latency、error log |

## Publication evidence

- `9434e9fd025bd8f019e257241a8e5de2ed973f3a` — delete inactive monitoring paths
- `1f4be5dc7fa9e9d11b9ba162b0e58395584954f6` — own CPAL streams on dedicated thread
- `528cad1e7ee7cd34fb53fa5829a18766c803bf21` — enforce model language capabilities
- `0259461e22e6858ce522f309a02f88f37ba7c014` — provision configured Breeze model
- `7a396930c88e7d27710ecc9819c1d1ae694418ae` — install Linux CPAL PipeWire dependencies
- `bff4cb91d41c3a668447d770eef8631d997d81b3` — expose live ASR benchmark adapter
- `4a97604e95ab72580896a80e175c0fe8cb568411` — enforce GPU-only inference
- `182b551d8b4b36758a54a5af321934383c08fe10` — publish GPU-enabled Linux builds
- `69e3e3c7668e3f668b02e20b49a015b2502067b7` — make CUDA release activation reproducible
- Remote：`JasonLn0711/meetily` `main`
- Post-push divergence against fork at implementation closeout：`0 0`

## Scope controls

- 本 audit 把前置 crash event、implementation、validation 與下一 gate 串成可追溯 evidence chain；原始 `source.log` 保持不變。
- Owner-thread architecture 與 25-cycle microphone lifecycle 已驗證；長時間雙音源與 hot-plug reliability 仍由 action ledger 管理。
- CUDA release runtime 已完成真實 zh-TW inference；Vulkan、Metal 與 HIP 的 live hardware qualification 各自由目標平台執行。
- Parakeet CUDA session policy 已實作；zh-TW 保持 model-language activation gate，不計入 live completed counts。
- LLM summary runtime 與 ASR 是不同 capability；本事件的 GPU-only policy 只管理 speech recognition inference。

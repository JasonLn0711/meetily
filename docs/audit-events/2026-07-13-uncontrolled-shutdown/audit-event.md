# Meetily 非預期終止 Audit Event

## 事件識別

- Event ID：`AUDIT-2026-07-13-MEETILY-UNCONTROLLED-SHUTDOWN-001`
- 事件時間：`2026-07-13T12:54:06Z` 至 `2026-07-13T12:54:19Z`，終止訊號緊接於最後一筆具時間戳記的 log 之後
- 臺灣時間：`2026-07-13 20:54:06` 至 `20:54:19`（Asia/Taipei）
- 事件類型：native heap corruption 導致應用程式非預期終止
- 目前狀態：`source preserved`、`analysis completed`、`implementation pending`
- 根因信心：高可信度推定；core dump / native backtrace 為下一驗證層
- Canonical home：Meetily execution repo

## 結論

應用程式由 glibc allocator 在偵測到 heap double-linked-list metadata 損壞後主動 abort。這次結束不屬於 Meetily 的受控停止錄音或 Tauri 正常退出流程；`[ELIFECYCLE] Command failed.` 是 pnpm 在子行程已異常結束後回報的外層結果。

最具行動價值的程式根因面位於 Linux audio lifecycle：錄音中的 device monitor 固定每 2 秒執行一次完整 CPAL/ALSA 裝置列舉，每次又重複掃描 ALSA input devices 與全部 devices。最後一輪 log 在 ALSA enumeration 訊息中途停止並立即出現 `corrupted double-linked list`，表示 allocator 很可能在這次 native audio 配置列舉／清理時偵測到先前或當下形成的 heap 損壞。

同一 ownership path 另有明確的安全風險：程式用 `unsafe impl Send` 強制讓含 `cpal::Stream` 的 `StreamBackend`、`AudioStream`、`AudioStreamManager` 與 `RecordingManager` 可跨執行緒移動，再把 live manager 存入全域 `Mutex<Option<RecordingManager>>`。CPAL 0.15.3 本身以 `NotSendSyncAcrossAllPlatforms` 明確移除 `Stream` 的 `Send` / `Sync`；目前註解所稱的 same-thread 保證並未由程式結構實作。這項 bypass 與高頻 native device enumeration 共同構成目前最高優先的修復範圍。

## 來源保存

- 完整原始 log：[source.log](source.log)
- 來源：使用者於 2026-07-13 提供的未刪節 terminal output
- 大小：73,837 bytes
- 行結構：891 個換行字元，892 個邏輯行
- SHA-256：`8a7c808ce5a0e436768659caeb677068b1f83fdac36ce01245bc447d38d74164`
- 保存方式：從本次本機 Codex session 的 user message 精確擷取三引號內文；未改寫、去重或刪除警告

## 觀測時間線

1. `12:54:06Z`：System 與 Microphone pipeline 持續處理 chunk 42000，resampler 正常輸出。
2. `12:54:10Z`：chunk 42100 正常完成。
3. `12:54:11Z`：VAD 完成 7.210 秒 speech segment；chunk 172 交由 Whisper CUDA runtime 處理。
4. `12:54:11Z`：Whisper 完成 chunk 172；`transcripts.json` 成功寫入 172 個 segments。
5. `12:54:15Z`：chunk 42200 正常完成。
6. `12:54:16Z`：incremental saver 成功保存 checkpoint 61，含 30 秒、1,440,000 samples。
7. `12:54:19Z`：chunk 42300 正常完成；這是最後一組應用層時間戳記。
8. 隨後：下一輪 ALSA/JACK enumeration 開始；輸出在兩筆 `pcm_oss` 訊息後中斷。
9. 終止：glibc 輸出 `corrupted double-linked list`，pnpm 接著輸出 `[ELIFECYCLE] Command failed.`。

## Log 訊號分類

### 致命訊號

- `corrupted double-linked list`：glibc malloc 的 consistency check 已發現 heap metadata 損壞。典型結果是 `SIGABRT`；訊息出現的位置可能是損壞被偵測的位置，而不一定是最初寫壞記憶體的位置。
- `[ELIFECYCLE] Command failed.`：套件管理器觀察到 Tauri/Rust 子行程非零結束；它是結果，不是根因。

### 高相關伴隨訊號

- `Cannot connect to server socket` / `jack server is not running`：49 組。裝置列舉持續探測 JACK，而現場未提供 JACK server。
- `pcm_oss.c ... /dev/dsp`：50 筆。列舉探測 legacy OSS PCM；`/dev/dsp` 在此環境不可用。
- `pcm_dmix.c`：392 筆；`pcm_dsnoop.c`：72 筆。完整 ALSA 掃描以 input/output 方向探測只支援單一方向的 plugin，產生大量 native open/close churn。
- 這些警告單獨存在時通常代表候選 backend/device 不可用；本事件中的關鍵是它們每 2 秒以完整批次重複，且最後一次 allocator abort 發生在該批次中途。

### 正常工作證據

- pipeline 在 13 秒觀測窗內完成 chunk 42000 至 42300。
- Whisper CUDA inference 完成並回傳文字。
- `transcripts.json` 與 checkpoint 61 均成功寫入。
- 因此 CUDA 初始化訊息、Whisper token trace、resampler RMS 百分比本身沒有直接提供 crash 證據。

## 程式碼證據

### 1. Device monitor 固定每 2 秒完整列舉

`frontend/src-tauri/src/audio/device_monitor.rs`：

- line 169 將 `check_interval` 固定為 2 秒。
- lines 178、184 每輪 sleep 後呼叫 `list_audio_devices()`。
- lines 241–249 計算 2 秒或 5 秒的 `next_interval`，但沒有把它指定回 `check_interval`；所謂 slower polling 實際未生效。

### 2. 每輪 Linux scan 重複進入 ALSA enumeration

`frontend/src-tauri/src/audio/devices/discovery.rs` 與 `platform/linux.rs`：

- `configure_linux_audio()` 先執行 `host.input_devices()`。
- 隨後以另一個 ALSA host 再執行一次 `input_devices()`。
- 回到 `list_audio_devices()` 後再執行 `host.devices()`，又掃描 input/output devices。
- 這條路徑可直接解釋 log 中週期性、成批出現的 JACK、OSS、dmix、dsnoop 訊息。

### 3. CPAL thread-safety contract 被 unsafe bypass

`frontend/src-tauri/src/audio/stream.rs` 與 `recording_manager.rs`：

- `StreamBackend` 包含 `cpal::Stream`，並在 line 28 宣告 `unsafe impl Send`。
- `AudioStream` 與 `AudioStreamManager` 也各自強制宣告 `Send`。
- `RecordingManager` 再強制宣告 `Send`。
- `recording_commands.rs` 把啟動後的 live manager 移入全域 `Mutex<Option<RecordingManager>>`，並由不同 Tauri async command / event callback 取得。
- CPAL 0.15.3 的 platform `Stream` 使用 `NotSendSyncAcrossAllPlatforms(PhantomData<*mut ()>)` 明確移除 `Send` 與 `Sync`。

### 4. 正常關閉證據未出現

正常停止錄音應先輸出：

- `🛑 Starting optimized recording shutdown...`
- `🚀 Using FORCE FLUSH...`
- `Stopping device monitor first...`
- `🎉 Recording stopped successfully...`

正常 Tauri app exit 應輸出：

- `Application exiting, cleaning up resources...`
- `Application cleanup complete`

原始 log 不含上述任何訊息；main window 的 close request 也被設計為 hide-to-tray。這支持「process abort 先於受控 cleanup」的判定。

## Dependency 與執行環境快照

- Git branch：`main`
- Git HEAD：`49b065fe1bff`
- Snapshot condition：工作樹已有使用者變更；`Cargo.toml`、`recording_manager.rs`、`stream.rs` 為 modified。本 audit 以事件分析當下的 working-tree code 為準。
- CPAL resolved version：`0.15.3`（registry package）
- OS：Ubuntu 24.04 family，Linux `6.17.0-35-generic`，x86_64
- glibc：`2.39-0ubuntu8.7`
- Rust：`1.95.0`
- pnpm：`11.11.0`
- Core evidence：本機未提供 `coredumpctl`；user journal 在事件時段未找到對應 crash backtrace。

CPAL 官方 0.17.0 release notes 明列修復 `ALSA: Data race in stream shutdown`。這項 upstream 證據提高目前 audio lifecycle/race 假設的優先度，也支持把 CPAL upgrade 納入驗證分支；它不取代本次缺少 core backtrace 的證據界線。

## 影響與已保存資料

- 應用程式在錄音中途被 native abort，當下 UI、audio capture、transcription worker 與受控 shutdown 流程一併停止。
- log 證實 `transcripts.json` 已保存 172 個 segments。
- log 證實 checkpoint 61 已保存 30 秒 audio buffer。
- 事件後是否仍有尚未寫入的 audio/transcript，需要用會議資料夾的檔案時間、metadata 與 checkpoint 序列另行核對。

## 根因判定

### Direct cause — confirmed

glibc 偵測 native heap double-linked-list corruption，並以 abort 結束 process。

### Primary actionable cause — high confidence

Linux 錄音期間的重複 CPAL/ALSA 全裝置列舉造成持續 native allocation/open/close 壓力；最後一次偵測點位於同一 enumeration 批次。此路徑應先停止高頻重掃並消除重複 enumeration。

### Ownership/race cause — high risk, validation pending

程式強制繞過 `cpal::Stream` 的 `!Send/!Sync` contract，live stream ownership 可能跨 Tauri/Tokio thread 移動。此設計可產生 undefined behavior，且現用 CPAL 版本早於官方列出的 ALSA stream-shutdown data-race 修復。

### Alternate native sources — retained for validation

Whisper/CUDA、resampler 或其他 FFI 也可能先寫壞 heap，之後由 ALSA allocation/free 偵測。現有 log 顯示 Whisper inference 已正常返回且程式再運作約 8 秒，因此它們目前排在 audio enumeration/ownership path 之後；core backtrace 或 sanitizer run 才能完成排除。

## 建議決策與下一驗證層

1. 先以最小 containment 停止「錄音期間每 2 秒完整掃描所有 ALSA PCM」；Linux device monitor 使用單一、方向正確的 device list，並讓穩定狀態的 interval 真正切換為較低頻率。
2. 將 `cpal::Stream` 保留在建立它的單一 owner thread，透過 channel 傳送 start/pause/stop 指令；移除目前四層 `unsafe impl Send` bypass。
3. 建立獨立 CPAL upgrade 驗證分支，至少納入官方已修復 ALSA shutdown race 的版本，再跑相同雙音源長時間錄音與裝置 hot-plug 測試。
4. 下一次重現前啟用 core dump 或以 AddressSanitizer / Valgrind 取得第一個 invalid write/free 的 native stack；保留完整 stdout/stderr、exit status、meeting artifact timestamps 與 dependency lockfile。
5. 驗收條件：長時間 mic + system audio 錄音期間沒有週期性全 ALSA 掃描；受控停止可完成全部 shutdown markers；重複 hot-plug 測試無 allocator abort，並保有 transcript/checkpoint 完整性。

## 外部依據

- GNU glibc bug record：相同 allocator 訊息對應 `SIGABRT`，maintainer 將其解釋為程式記憶體損壞的高機率結果：<https://sourceware.org/pipermail/glibc-bugs/2018-August/043085.html>
- CPAL 0.17.0 official release：明列修復 `ALSA: Data race in stream shutdown`：<https://github.com/RustAudio/cpal/releases/tag/v0.17.0>

## Scope controls

- 本 audit 完成來源保存、程式路徑分析與修復優先序。
- 本次未變更 production code，也未宣稱已重現或已修復。
- 事件的 direct termination cause 已由 log 證實；第一個造成 heap 損壞的 native instruction 仍由 core/sanitizer validation layer 確認。

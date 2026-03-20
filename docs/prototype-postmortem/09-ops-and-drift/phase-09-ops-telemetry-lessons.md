# Phase 09 — Operational Telemetry and Supportability Lessons

**Purpose**: Extract logging architecture lessons from the GUI prototype, document the alpha bug-reporting process as a product requirement, and define the minimum support telemetry baseline for the Rust successor.

---

## 1. Logging Architecture — What Was Built

### 1.1 Scope

The logging system was implemented entirely in `conset-pdf-gui`. The core library (`conset-pdf`) has no structured logging — it uses `console.log/warn/error` directly. The GUI introduced a proper logging layer as part of the alpha release.

### 1.2 Component Map

| File | Role |
|------|------|
| `src/main/utils/logger.ts` | Core logger: file rotation, session tracking, performance metrics |
| `src/main/utils/system-info.ts` | System metadata collector for bug reports |
| `src/main/ipc/logging.ts` | IPC handlers: renderer→main log forwarding and log management |
| `src/modules/logger.js` | Renderer-side logger wrapper (`window.logger`) |
| `src/preload.ts` | Exposes `window.wizardAPI.log.*` to renderer |

### 1.3 Log Levels

| Level | Semantics | Default in Production |
|-------|----------|----------------------|
| `debug` | Detailed troubleshooting | OFF |
| `info` | Normal operations, session lifecycle | ON |
| `warn` | Unusual but non-fatal | ON |
| `error` | Failures with stack traces | ON |

### 1.4 Persistence Constraints (Concrete)

| Constraint | Value |
|-----------|-------|
| Max file size | **10 MB per file** |
| Max files retained | **5 files** |
| Storage location (Windows) | `%APPDATA%\Conset PDF\logs\` |
| Storage location (macOS) | `~/Library/Application Support/Conset PDF/logs/` |
| Write strategy | Async batched writes (non-blocking) |

The 5 × 10 MB = 50 MB maximum log footprint was chosen as acceptable for an alpha desktop app. For a V4 Tauri/Rust binary, `tracing-appender` with `RollingFileAppender` can implement equivalent rotation.

**Note from Phase 8**: The earlier verification found constraints of 5 × 2 MB in one doc; the implementation summary (`LOGGING_IMPLEMENTATION.md`) says 10 MB per file. The **implementation spec** (10 MB) takes precedence over any other mention.

### 1.5 Session Lifecycle Markers

Session markers are logged at startup:
```typescript
// main.ts: logger initialized at startup, session ID generated
logger.info('App', 'Session start', { sessionId, systemInfo });
```

Session end is not explicitly marked (process exit). This is a gap: for Rust, session end should be explicitly logged with duration.

### 1.6 Unhandled Error Capture

Both processes capture unhandled errors automatically:

**Main process** (`main.ts`):
```typescript
process.on('uncaughtException', error => logger.error(...));
process.on('unhandledRejection', error => logger.error(...));
```

**Renderer process** (`modules/logger.js`):
```javascript
window.addEventListener('error', event => ...);
window.addEventListener('unhandledrejection', event => ...);
```

This means any crash or unhandled promise rejection is guaranteed to appear in the log file before process exit (modulo async flush timing). For Rust, `std::panic::set_hook` is the equivalent pair for unhandled panics.

### 1.7 Performance Logging API

The logger exposes a timer API for operation timing:
```javascript
const timer = window.logger.startTimer('Component', 'operation-name');
// ... do work ...
timer.end({ additionalContext: value });
```

This produces structured performance entries. The design patterns to carry forward:
- Every major operation (merge, analyze, extract) should emit timing
- Context metadata (page count, file size, detection source) should accompany timing entries

### 1.8 Export Bundle for Bug Reports

The log export bundle contains:
1. All log files from current and recent sessions
2. System information (app version, OS, CPU, memory, Electron/Node versions, display config)
3. Timestamps and context for every entry

The export is triggered from the Settings UI: **Settings → Debug Logging → Export All Logs**.

This is the support handoff artifact. A Rust/Tauri successor must provide equivalent export capability.

---

## 2. IPC Logging Channel Contracts

The following IPC channels were added for log management (from `src/main/ipc/logging.ts`):

| Channel | Direction | Purpose |
|---------|-----------|---------|
| `log:debug` / `log:info` / `log:warn` / `log:error` | Renderer→Main | Forward renderer log entries to main-process log file |
| `log:performance` | Renderer→Main | Record performance metrics |
| `log:getFiles` | Renderer→Main | List all log files |
| `log:readFile` | Renderer→Main | Read specific log file |
| `log:export` | Renderer→Main | Export all logs with system info |
| `log:openDirectory` | Renderer→Main | Open log folder in OS file explorer |
| `log:clear` | Renderer→Main | Delete all log files |
| `log:getStats` | Renderer→Main | Get log statistics (file count, total size, directory) |
| `log:getSystemInfo` | Renderer→Main | Get system metadata |

All channels follow the standard `IpcResponse<T>` envelope.

---

## 3. Alpha Bug-Reporting Process

### 3.1 What Was Established

The alpha release included:
- `docs/ALPHA_TESTING_GUIDE.md` — user-facing bug reporting guide
- `docs/PRE_ALPHA_CHECKLIST.md` — pre-release verification checklist
- Placeholders in the guide for contact info, issue tracker URL, and shared drive URL (not filled at extraction time)

### 3.2 Bug Report Template (from ALPHA_TESTING_GUIDE.md)

```
Bug Report: [Short title]
Description: [What happened?]
Expected Behavior: [What should have happened?]
Steps to Reproduce: [numbered steps]
System Information: [OS, app version, date/time]
Files: [log file, screenshot]
Additional Notes: [anything else]
```

### 3.3 Severity Classification (from PRE_ALPHA_CHECKLIST.md)

| Severity | Definition | Response |
|---------|-----------|---------|
| Critical | App crash, data loss | Fix immediately |
| High | Major feature broken | Fix in next build |
| Medium | Minor bug, workaround available | Schedule for fixes |
| Low | Cosmetic, enhancement request | Future consideration |

### 3.4 Key Operational Constraints for Alpha

- Logs must be exported immediately after a bug — rotation may overwrite older sessions
- The pre-alpha checklist verified: 5 × 10 MB rotation, export works, unhandled errors logged, startup time < 5 seconds
- The alpha guide instructs testers to export logs before restarting the app

---

## 4. Minimum Support Telemetry Baseline for Rust Successor

These are the **non-negotiable minimum** telemetry behaviors for the V4 Rust/Tauri successor, derived from alpha constraints and lessons:

### 4.1 Startup and Session

| Requirement | Rationale |
|-------------|-----------|
| Emit session-start log entry with: session ID, app version, OS version, memory, CPU count | Required to correlate logs across files; bug reports that don't include session info are undiagnosable |
| Emit session-end log entry with: session duration | Provides baseline timing; helps differentiate crashes from clean exits |
| Session ID must be stable within a process lifetime, but unique per launch | Prevents cross-session log confusion |

### 4.2 Operation Timing

| Requirement | Rationale |
|-------------|-----------|
| Every top-level user operation must emit: operation name, start time, end time, success/failure, input size | Performance regression detection; reproducibility context |
| Timing context must include: `pageCount`, `fileSize` where applicable | Correlates performance to input scale |

### 4.3 Failure Context

| Requirement | Rationale |
|-------------|-----------|
| Every error must include: module name, operation name, error message, stack trace or Rust backtrace | Stack traces are the single most valuable bug report artifact |
| Unhandled panics must be captured before unwinding: `std::panic::set_hook` | Ensures panics appear in log before process terminates |
| IPC errors (in Tauri context) must include the channel name and input payload type | IPC failures are systematically hard to diagnose without this |

### 4.4 Reproducibility Bundle

| Requirement | Rationale |
|-------------|-----------|
| Export bundle must contain: all session log files + system info JSON | Directly replicates what was proven valuable in alpha |
| System info must include: app version, OS, memory (total/free), CPU, display count/resolution | Required to reproduce environment-specific failures |
| Export must be triggerable from Settings UI without navigating log file paths | Alpha testers struggle with file system navigation |

### 4.5 Log Storage

| Requirement | Value |
|-------------|-------|
| Rotation policy | N files × M MB (recommend ≥ 5 files × ≥ 5 MB for production) |
| Encoding | UTF-8 only |
| Format | Structured line format: `<timestamp> [LEVEL] [Module] message {json-context}` |
| Location | OS-standard user data directory (not application install directory) |

### 4.6 Privacy Requirements

| Requirement | Rationale |
|-------------|-----------|
| Log files must never contain PDF content or extracted text | AEC documents contain PII and proprietary project data |
| File paths in logs must be logged as configurable (allow path masking for sensitive projects) | Some users may have confidential project paths |
| System info in export bundles must not include user account names | GDPR/CCPA compliance baseline |

---

## 5. Lessons Learned from Logging Implementation

### What Worked
- **Dual-process logging via IPC** was seamless. Renderer logs were forwarded to the main-process file with negligible overhead. The structured `window.logger` API was easy for UI developers to adopt.
- **Export bundle** was the most valuable alpha support tool. Without it, remote debugging would be impractical.
- **Automatic error capture** (uncaught exceptions + unhandled rejections) caught bugs that would otherwise have required user ability to open DevTools.

### What Didn't Work / Gaps
- **No session-end marker**: Session duration is not captured. Bug reports can't distinguish clean exit from crash.
- **No structured operation events**: Individual log calls are narrative (English messages), not structured events. Performance analysis requires text parsing rather than query.
- **Core library has no logger**: `conset-pdf` uses `console.log`. In the CLI, these go to stdout. In the GUI context, they bypass the structured log file entirely. A Rust core crate should use `tracing` crate from day one and configure subscribers per deployment context.
- **Log file size check was aspirational**: PRE_ALPHA_CHECKLIST had "check log file sizes stay under 10MB before rotation" as a manual step. Rotation is automatic but there's no automated test asserting the behavior.
- **No sampling or rate limiting**: All debug/info events are logged. For high-frequency operations (page iteration in a 500-page merge), this could produce 10k+ log entries in a single operation. Rust successor should implement event sampling for high-frequency inner loops.

---

## 6. `tracing` Crate Recommendations for Rust Successor

| Rust Component | Recommendation |
|---------------|----------------|
| Core logging abstraction | `tracing` crate (structured, zero-cost spans) |
| File output | `tracing-appender` with `RollingFileAppender` |
| JSON format | `tracing-subscriber` with `json()` layer |
| Panic hook | `std::panic::set_hook` → log to `tracing::error!` before abort |
| Session ID | Generate at startup with `uuid` crate, inject into every span |
| Performance spans | `tracing::instrument` macro on all workflow entry points |

The `tracing` crate's instrumentation macro approach (`#[instrument]`) is far superior to the prototype's manual `startTimer()`/`timer.end()` pattern and should be the default.

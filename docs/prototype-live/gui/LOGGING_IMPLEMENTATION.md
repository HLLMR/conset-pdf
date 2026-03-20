# Logging Implementation Summary

This document summarizes the comprehensive logging and debug reporting system added to Conset PDF GUI.

## What Was Implemented

### 1. Core Logger Module (`src/main/utils/logger.ts`)
A robust logging system for the main process with:
- ✅ Multi-level logging: debug, info, warn, error
- ✅ Persistent file storage with automatic rotation
- ✅ Log files limited to 10MB each, keeping up to 5 files
- ✅ Asynchronous batched writes for performance
- ✅ Performance metrics tracking
- ✅ Session tracking with unique IDs
- ✅ Automatic cleanup of old logs
- ✅ Export functionality for debugging
- ✅ Configurable log levels based on environment

### 2. System Information Collector (`src/main/utils/system-info.ts`)
Captures comprehensive system information for bug reports:
- ✅ Application version and metadata
- ✅ Electron, Node.js, and Chrome versions
- ✅ Operating system details
- ✅ CPU information
- ✅ Memory usage (total and free)
- ✅ Display configuration (resolution, scale, multi-monitor)
- ✅ System uptime

### 3. IPC Handlers (`src/main/ipc/logging.ts`)
Registers IPC handlers for renderer-to-main communication:
- ✅ `log:debug`, `log:info`, `log:warn`, `log:error` - Log from renderer
- ✅ `log:performance` - Record performance metrics
- ✅ `log:getFiles` - List all log files
- ✅ `log:readFile` - Read specific log file content
- ✅ `log:export` - Export all logs with system info
- ✅ `log:openDirectory` - Open log folder in file explorer
- ✅ `log:clear` - Clear all logs
- ✅ `log:getStats` - Get logging statistics
- ✅ `log:getSystemInfo` - Get system information

### 4. Renderer Logger (`src/modules/logger.js`)
Simple API for logging from UI code:
- ✅ `window.logger.debug()`, `.info()`, `.warn()`, `.error()`
- ✅ `window.logger.startTimer()` - Create performance timers
- ✅ `window.logger.wrapTimed()` - Wrap functions with timing
- ✅ `window.logger.captureErrors()` - Automatic error capture
- ✅ Auto-initialization on app load

### 5. Preload API Updates (`src/preload.ts`)
Exposed logging API to renderer process:
- ✅ Complete logging API in `window.wizardAPI.log.*`
- ✅ Type-safe error serialization
- ✅ Consistent IPC response handling

### 6. Settings UI (`src/app.html`, `src/settings-view.js`)
User interface for log management:
- ✅ Log statistics dashboard (file count, total size, directory)
- ✅ **Refresh Stats** button - Updates statistics
- ✅ **Export All Logs** button - Saves combined log file with system info
- ✅ **Open Log Folder** button - Opens logs in file explorer
- ✅ **Clear All Logs** button - Removes all log files
- ✅ Expandable log files list with view buttons
- ✅ In-app log viewer for individual files

### 7. Main Process Integration (`src/main.ts`)
- ✅ Logger imported and initialized
- ✅ Replaced console.log/error with logger calls
- ✅ Automatic capture of uncaught exceptions
- ✅ Automatic capture of unhandled promise rejections
- ✅ Session start logging with system info

### 8. Documentation

#### For Developers (`docs/LOGGING.md`)
- ✅ Complete API reference
- ✅ Usage examples for main and renderer processes
- ✅ Configuration options
- ✅ Best practices
- ✅ Troubleshooting guide

#### For Alpha Testers (`docs/ALPHA_TESTING_GUIDE.md`)
- ✅ Step-by-step bug reporting instructions
- ✅ How to export logs
- ✅ Bug report template
- ✅ FAQ section
- ✅ Privacy information

## File Structure

```
conset-pdf-gui/
├── src/
│   ├── main.ts                        # Logger initialized here
│   ├── preload.ts                     # API exposed to renderer
│   ├── app.html                       # Logger UI added to settings
│   ├── settings-view.js               # Logger UI logic
│   ├── main/
│   │   ├── ipc/
│   │   │   ├── index.ts              # Logger handlers registered
│   │   │   └── logging.ts            # NEW: IPC handlers for logging
│   │   └── utils/
│   │       ├── logger.ts             # NEW: Core logger implementation
│   │       └── system-info.ts        # NEW: System info collector
│   └── modules/
│       └── logger.js                 # NEW: Renderer logger wrapper
└── docs/
    ├── LOGGING.md                    # NEW: Developer documentation
    └── ALPHA_TESTING_GUIDE.md        # NEW: Tester documentation
```

## Log File Location

Logs are stored in the application's user data directory:
- **Windows**: `%APPDATA%\Conset PDF\logs\`
- **macOS**: `~/Library/Application Support/Conset PDF/logs/`
- **Linux**: `~/.config/Conset PDF/logs/`

## Log Format

```
2026-03-02T10:30:45.123Z [INFO] [MyModule] Operation completed {"key":"value"}
2026-03-02T10:30:46.456Z [ERROR] [MyModule] Operation failed
Stack: Error: Something went wrong
    at MyModule.doSomething (file.js:42:10)
```

## Usage Examples

### Main Process (TypeScript)

```typescript
import { logger } from './main/utils/logger.js';

// Basic logging
logger.info('PDFHandler', 'Loaded PDF successfully', { pages: 42 });
logger.error('PDFHandler', 'Failed to load PDF', error);

// Performance
const start = Date.now();
// ... do work ...
logger.performance('PDFHandler', 'render-thumbnail', Date.now() - start);
```

### Renderer Process (JavaScript)

```javascript
// Simple logging
window.logger.info('BookmarkWizard', 'User clicked next');
window.logger.error('BookmarkWizard', 'Validation failed', validationError);

// Performance timing
const timer = window.logger.startTimer('BookmarkWizard', 'generate-bookmarks');
await generateBookmarks();
timer.end({ bookmarkCount: 25 });
```

## How Alpha Testers Submit Logs

1. Open **Settings** (⚙️ icon)
2. Scroll to **Debug Logging**
3. Click **Export All Logs**
4. Save file (automatically includes system info)
5. Attach to bug report email/ticket

## Configuration

The logger adapts to environment:

| Environment | Log Level | Console Output | File Output |
|------------|-----------|----------------|-------------|
| Development | `debug` | ✅ Yes | ✅ Yes |
| Production | `info` | ❌ No | ✅ Yes |

Can be overridden programmatically:

```typescript
logger.configure({
  logLevel: 'warn',        // Only warn and error
  enableConsole: false,    // Disable console output
  maxFiles: 10,            // Keep more files
});
```

## Testing the Implementation

### Manual Testing

1. **Build the app:**
   ```bash
   cd conset-pdf-gui
   npm run build
   npm start
   ```

2. **Generate some logs:**
   - Navigate through different views
   - Perform some operations (load PDFs, run wizards, etc.)
   - Intentionally trigger an error (if possible)

3. **View logs:**
   - Go to Settings → Debug Logging
   - Click "Refresh Stats"
   - Verify file count and size are shown
   - Click "View" on a log file
   - Check that log content appears

4. **Export logs:**
   - Click "Export All Logs"
   - Open the exported file
   - Verify it contains:
     - System information header
     - All log files
     - Properly formatted entries

5. **Open log directory:**
   - Click "Open Log Folder"
   - Verify folder opens in file explorer
   - Check that log files exist

6. **Clear logs:**
   - Click "Clear All Logs"
   - Confirm the action
   - Verify stats show 1 file (new session)

### Automated Testing

Add tests for logger functionality:

```typescript
// Example test (add to tests/logging/)
describe('Logger', () => {
  it('should create log file', async () => {
    logger.info('Test', 'Test message');
    await logger.flush();
    const files = await logger.getLogFiles();
    expect(files.length).toBeGreaterThan(0);
  });
});
```

## Future Enhancements

Potential improvements for later versions:

- [ ] Automatic log upload to remote server
- [ ] Email integration for sending logs
- [ ] Real-time log streaming in dev mode
- [ ] Log filtering/searching in UI
- [ ] Structured logging with queryable metadata
- [ ] Log compression for archival
- [ ] User consent dialog for telemetry
- [ ] Anonymous crash reporting

## Rollback

If this implementation causes issues, you can temporarily disable it:

1. Comment out logger initialization in `main.ts`:
   ```typescript
   // import { logger } from './main/utils/logger.js';
   ```

2. Comment out logger module import in `app.html`:
   ```html
   <!-- <script type="module" src="modules/logger.js"></script> -->
   ```

3. Rebuild:
   ```bash
   npm run build
   ```

The app will fall back to console logging.

## Support

For questions or issues with the logging system:

- Review documentation in `docs/LOGGING.md`
- Check the implementation files listed above
- Test in development mode with `enableConsole: true`
- Check that log directory is writable

## Summary

This implementation provides:

✅ **Complete logging** from main and renderer processes  
✅ **Persistent storage** with automatic rotation and cleanup  
✅ **Easy export** for alpha testers to submit bug reports  
✅ **System information** included automatically in exports  
✅ **User-friendly UI** in Settings for log management  
✅ **Performance tracking** for optimization  
✅ **Automatic error capture** for unhandled exceptions  
✅ **Comprehensive documentation** for developers and testers  
✅ **Production-ready** with environment-based configuration  

The logging system is now ready for alpha testing and will significantly improve your ability to debug issues reported by testers!

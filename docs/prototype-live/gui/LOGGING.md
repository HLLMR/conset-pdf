# Logging and Debug Reporting Guide

This document describes the logging system implemented in Conset PDF GUI and how to use it for debugging and error reporting.

## Overview

The application includes a comprehensive logging system that:

- Captures logs from both main and renderer processes
- Persists logs to disk with automatic rotation
- Provides multiple log levels (debug, info, warn, error)
- Records performance metrics
- Allows easy export for bug reporting
- Captures unhandled errors and promise rejections automatically

## Log Levels

- **debug**: Detailed information for troubleshooting (only in development by default)
- **info**: General informational messages
- **warn**: Warning messages that don't prevent operation
- **error**: Error messages with stack traces

## For Developers

### Main Process (TypeScript/Node.js)

Import the logger:

```typescript
import { logger } from './main/utils/logger.js';
```

Usage examples:

```typescript
// Basic logging
logger.info('MyModule', 'Operation completed successfully');
logger.warn('MyModule', 'Something unusual happened', { detail: 'value' });
logger.error('MyModule', 'Operation failed', error);

// Performance logging
const startTime = Date.now();
// ... do work ...
logger.performance('MyModule', 'process-document', Date.now() - startTime, {
  pageCount: 42,
  fileSize: 1024000
});

// Debug logging (only visible in dev mode)
logger.debug('MyModule', 'Detailed state', { state: currentState });
```

### Renderer Process (JavaScript)

The logger is automatically available as `window.logger`:

```javascript
// Basic logging
window.logger.info('MyComponent', 'User clicked button');
window.logger.warn('MyComponent', 'Validation warning', { field: 'name' });
window.logger.error('MyComponent', 'Failed to save', error);

// Performance timing
const timer = window.logger.startTimer('MyComponent', 'render-chart');
// ... do work ...
timer.end({ chartType: 'bar', dataPoints: 1000 });

// Or wrap an async function
const timedFetch = window.logger.wrapTimed('DataLoader', 'fetch-data', async () => {
  return await fetch('...');
});
await timedFetch();
```

### Automatic Error Capture

Unhandled errors and promise rejections are automatically captured in both processes:

**Main Process:**
```typescript
// Automatically logged
process.on('uncaughtException', ...);
process.on('unhandledRejection', ...);
```

**Renderer Process:**
```javascript
// Automatically logged (set up in modules/logger.js)
window.addEventListener('error', ...);
window.addEventListener('unhandledrejection', ...);
```

### Configuration

The logger can be configured programmatically:

```typescript
import { logger } from './main/utils/logger.js';

logger.configure({
  logLevel: 'debug',           // Minimum level to log
  maxFileSize: 10 * 1024 * 1024, // 10 MB per file
  maxFiles: 5,                  // Keep up to 5 log files
  enableConsole: true,          // Also output to console
});
```

Default configuration:
- **Development**: log level = `debug`, console enabled
- **Production**: log level = `info`, console disabled (file only)

## For Alpha Testers

### Viewing Logs

1. Open the application
2. Navigate to **Settings** (gear icon in left navigation)
3. Scroll to the **Debug Logging** section
4. Click **Refresh Stats** to view current log statistics

### Exporting Logs for Bug Reports

When reporting a bug:

1. Go to **Settings** → **Debug Logging**
2. Click **Export All Logs**
3. Choose a location to save the export file
4. Attach the exported file to your bug report via email or issue tracker

The exported file contains:
- All log files from the current session and recent sessions
- App version and system information
- Timestamps and context for every log entry

### Opening Log Directory

To manually inspect log files:

1. Go to **Settings** → **Debug Logging**
2. Click **Open Log Folder**
3. Your file explorer will open the logs directory

### Clearing Logs

To free up space or start fresh:

1. Go to **Settings** → **Debug Logging**
2. Click **Clear All Logs**
3. Confirm the action (this cannot be undone)

Note: Clearing logs will start a new logging session immediately.

## Log File Location

Logs are stored in the application's user data directory:

- **Windows**: `%APPDATA%/Conset PDF/logs/`
- **macOS**: `~/Library/Application Support/Conset PDF/logs/`
- **Linux**: `~/.config/Conset PDF/logs/`

## Log File Format

Log files are plain text with the following format:

```
2026-03-02T10:30:45.123Z [INFO] [MyModule] Operation completed {"detail":"value"}
2026-03-02T10:30:46.456Z [ERROR] [MyModule] Operation failed
Stack: Error: Something went wrong
    at MyModule.doSomething (file.js:42:10)
    ...
```

Each line contains:
1. ISO 8601 timestamp
2. Log level in brackets
3. Context/module name in brackets
4. Message
5. Optional JSON data
6. Optional stack trace (for errors)

## Log Rotation

Logs automatically rotate when a file reaches 10 MB. The system keeps up to 5 log files, deleting the oldest when the limit is reached.

File naming: `app-YYYY-MM-DDTHH-MM-SS.log`

## Best Practices

### Do Log:
- Major operations starting/completing
- User actions (clicks, selections, inputs)
- File operations (open, save, import, export)
- API calls and responses
- Performance metrics for slow operations
- Errors with full context and stack traces
- State transitions

### Don't Log:
- Sensitive user data (passwords, API keys, personal information)
- Excessive detail in tight loops (use debug level sparingly)
- Binary data or very large objects
- Redundant information

### Naming Contexts

Use clear, hierarchical context names:
- `Renderer:WizardShell` for renderer process modules
- `ProfileStore` for main process modules
- `IPC:Operations` for IPC handlers
- `Renderer:BookmarkWizard` for specific UI components

## Troubleshooting

### Logs not appearing
1. Check that the app has write permissions to the user data directory
2. Verify log level is appropriate (debug logs won't show in production unless configured)
3. Check that logger is properly imported/initialized

### Log files too large
- Adjust `maxFileSize` configuration if needed
- Reduce number of `maxFiles` kept
- Clear old logs periodically
- Review debug-level logging in production code

### Performance impact
- File writes are batched and asynchronous
- Minimal performance impact under normal use
- If concerned, adjust log level to `warn` or `error` only

## Future Enhancements

Potential improvements for future versions:

- [ ] Automatic log submission to central server
- [ ] Log filtering/searching in UI
- [ ] Real-time log viewer
- [ ] Email integration for sending logs
- [ ] Structured logging with queryable metadata
- [ ] Log compression for archival
- [ ] Remote logging for unattended systems

## Support

If you encounter issues with the logging system itself, please report them with:
- Your operating system version
- Application version
- Steps to reproduce
- Any error messages displayed

For questions, contact the development team.

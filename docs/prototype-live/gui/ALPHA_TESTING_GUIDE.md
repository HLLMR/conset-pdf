# Alpha Testing: Bug Reporting Guide

Thank you for helping test Conset PDF! This guide explains how to report bugs effectively.

## When to Report a Bug

Report any issues you encounter:

- Application crashes or freezes
- Incorrect results or outputs
- UI elements not working as expected
- Performance problems (slow operations)
- Error messages or warnings
- Confusing or unclear behavior
- Missing features you expected

## How to Report a Bug

### 1. Prepare Your Report

Before reporting, gather the following information:

- **What you were trying to do** (e.g., "I was trying to merge a drawing set")
- **What you expected to happen** (e.g., "I expected the drawings to be combined into one PDF")
- **What actually happened** (e.g., "The app froze and stopped responding")
- **Steps to reproduce** (What exact steps led to the problem?)

### 2. Export Debug Logs

**This is the most important step!** Logs help developers understand what went wrong.

1. Open Conset PDF
2. Click the **Settings** button (⚙️ gear icon) in the left navigation
3. Scroll down to the **Debug Logging** section
4. Click **Refresh Stats** to load current log information
5. Click **Export All Logs**
6. Save the file to your Desktop (default location)
7. Note the filename (e.g., `conset-pdf-logs-2026-03-02.txt`)

**Important:** Try to export logs soon after the bug occurs. If you close and reopen the app multiple times, newer logs may overwrite older ones.

### 3. Take Screenshots (if applicable)

If the bug is visual or related to the UI:

1. Press `Win + Shift + S` (Windows) or `Cmd + Shift + 4` (macOS) to capture a screenshot
2. Highlight the problem area
3. Save the screenshot

### 4. Submit Your Report

Send your bug report via [INSERT YOUR PREFERRED METHOD]:

**Option A: Email**
- Email: [INSERT EMAIL ADDRESS]
- Subject: `[Bug] Brief description`
- Attach: Exported log file and screenshots

**Option B: Issue Tracker**
- URL: [INSERT ISSUE TRACKER URL]
- Create a new issue
- Attach: Exported log file and screenshots

**Option C: Shared Drive**
- Upload to: [INSERT SHARED DRIVE URL]
- Name your folder: `Bug_[YourName]_[Date]`
- Include log file, screenshots, and a description.txt file

### 5. Bug Report Template

Copy and fill out this template:

```
Bug Report: [Short title]

Description:
[What happened? Be specific]

Expected Behavior:
[What should have happened?]

Steps to Reproduce:
1. [First step]
2. [Second step]
3. [...]

System Information:
- Windows/macOS/Linux: [Your OS]
- Conset PDF Version: [Check Help → About, or Settings]
- Date/Time of Issue: [When did it happen?]

Files:
- Attached log file: [filename]
- Attached screenshot: [filename (if applicable)]

Additional Notes:
[Anything else that might be helpful]
```

## Understanding Log Levels

You may see messages like these in the application:

- 🟦 **INFO**: Normal operation (e.g., "File loaded successfully")
- 🟨 **WARN**: Something unusual but not critical (e.g., "File format slightly off, but we handled it")
- 🟥 **ERROR**: Something failed (e.g., "Could not save file")

Report all errors, and mention if you see frequent warnings.

## Viewing Logs Yourself

If you're curious or want to debug on your own:

### Open Log Folder
1. Go to **Settings** → **Debug Logging**
2. Click **Open Log Folder**
3. View files with any text editor (Notepad, VS Code, etc.)

### View Specific Log File
1. Go to **Settings** → **Debug Logging**
2. Click **Refresh Stats**
3. Expand **Log Files** section
4. Click **View** next to any log file
5. Read the content in the viewer below

### What Logs Contain

Log files show:
- Every major operation performed
- Errors with detailed stack traces
- Performance timings
- System information

**Privacy Note:** Logs do NOT contain:
- Passwords
- Personal API keys
- File contents (only paths and metadata)
- Sensitive information

## Frequently Asked Questions

### Q: Will logs slow down the app?
**A:** No. Logging is asynchronous and batched. You won't notice any performance impact.

### Q: How much disk space do logs use?
**A:** Each log file can grow up to 10 MB. The app keeps a maximum of 5 files, so at most 50 MB total.

### Q: Can I delete old logs?
**A:** Yes! Go to **Settings** → **Debug Logging** → **Clear All Logs**. This removes all old logs immediately.

### Q: Do I need to export logs every time?
**A:** No, only when reporting a bug. Normal usage doesn't require any action.

### Q: What if the app crashes before I can export logs?
**A:** Logs are written to disk continuously, so they're preserved even after a crash. Just restart the app and export them from Settings.

### Q: Can I look at logs from a previous session?
**A:** Yes! Multiple log files are kept. When you export logs, ALL recent sessions are included in one file.

## Need Help?

If you have questions about testing or reporting bugs:

- Contact: [INSERT CONTACT INFO]
- Documentation: Check the `docs/` folder in the installation directory
- Community: [INSERT COMMUNITY FORUM/CHAT LINK]

---

**Thank you for helping make Conset PDF better!** Your feedback is invaluable. 🎉

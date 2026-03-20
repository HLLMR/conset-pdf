# Pre-Alpha Release Checklist

Use this checklist before distributing your alpha build to testers.

## Build Verification

- [ ] Run `npm run build` in `conset-pdf-gui` - completes without errors
- [ ] Run `npm run build:win` (or platform-specific) - creates installer
- [ ] Test the packaged app launches successfully
- [ ] Verify app version displays correctly in About/Settings

## Logging System Verification

- [ ] Logging UI appears in Settings view
- [ ] "Refresh Stats" button works and displays log information
- [ ] "Export All Logs" button creates a file successfully
- [ ] Exported file contains:
  - [ ] System information header
  - [ ] All log file contents
  - [ ] Readable format
- [ ] "Open Log Folder" button opens correct directory
- [ ] Log files exist in user data directory
- [ ] Logs contain timestamps, levels, contexts, and messages
- [ ] "Clear All Logs" button removes files and restarts logging

## Error Handling

- [ ] Intentionally trigger an error - verify it's logged
- [ ] Check that stack traces appear in error logs
- [ ] Verify unhandled promise rejections are captured
- [ ] Test that the app continues running after logging an error

## Performance

- [ ] App startup time is acceptable (< 5 seconds)
- [ ] UI remains responsive during normal operations
- [ ] Log writes don't cause visible lag
- [ ] Check log file sizes stay under 10MB before rotation
- [ ] Verify only 5 most recent log files are kept

## Documentation

- [ ] Alpha testing guide exists: `docs/ALPHA_TESTING_GUIDE.md`
- [ ] Update guide with your actual contact info:
  - [ ] Replace `[INSERT EMAIL ADDRESS]` with your email
  - [ ] Replace `[INSERT ISSUE TRACKER URL]` with your tracker URL  
  - [ ] Replace `[INSERT SHARED DRIVE URL]` if applicable
  - [ ] Replace `[INSERT CONTACT INFO]` with support contact
  - [ ] Replace `[INSERT COMMUNITY FORUM/CHAT LINK]` if applicable
- [ ] Include testing guide in distribution (e.g., as PDF or in README)

## Distribution Package

- [ ] Create distribution folder/archive containing:
  - [ ] Installer or executable
  - [ ] README with quick start instructions
  - [ ] Alpha testing guide (PDF or Markdown)
  - [ ] Contact information
  - [ ] Known issues list (if any)
  - [ ] License information
- [ ] Test installation on a clean machine (if possible)
- [ ] Verify installer doesn't require admin rights (or clearly states if it does)

## Communication with Testers

- [ ] Prepare welcome email/message with:
  - [ ] Installation instructions
  - [ ] Link to full testing guide
  - [ ] Expected testing period/timeline
  - [ ] How to submit feedback and bugs
  - [ ] Your availability for support
  - [ ] Any known limitations or WIP features
- [ ] Set up bug tracking system:
  - [ ] Email inbox dedicated to bug reports
  - [ ] OR issue tracker (GitHub, Jira, etc.)
  - [ ] OR shared folder for submissions
- [ ] Prepare response templates:
  - [ ] "Thank you for the bug report"
  - [ ] "Please export logs and send them"
  - [ ] "Fixed in next build"

## Testing Instructions for Alpha Testers

Include these key points in your welcome message:

### What to Test
- [ ] Normal workflows (list your main features)
- [ ] Edge cases (empty files, large files, unusual inputs)
- [ ] Performance (slow operations, responsiveness)
- [ ] UI/UX clarity (confusing elements, missing labels)

### How to Report Bugs
1. **Always export logs** (Settings → Debug Logging → Export All Logs)
2. Describe what you were doing
3. What you expected vs. what happened
4. Include screenshots if relevant
5. Attach exported log file

### Important Reminders
- This is alpha software - expect bugs!
- Save your work frequently
- Don't use on critical production files without backups
- Contact us immediately if you see data loss or corruption

## Log Management During Alpha

- [ ] Check disk space on alpha test machines regularly
- [ ] Consider setting up log aggregation (optional for small alpha)
- [ ] Review submitted logs to identify common issues
- [ ] Create FAQ based on common questions/bugs

## Post-Distribution

After sending to testers:

- [ ] Track who has received the build
- [ ] Monitor for incoming bug reports
- [ ] Respond to testers within 24-48 hours
- [ ] Triage bugs by severity:
  - **Critical**: App crashes, data loss → Fix immediately
  - **High**: Major features broken → Fix in next build
  - **Medium**: Minor bugs, workarounds available → Schedule for fixes
  - **Low**: Cosmetic, enhancement requests → Future consideration
- [ ] Keep a changelog of fixes for next alpha version

## Sample Welcome Email Template

```
Subject: Conset PDF Alpha Testing - Welcome!

Hi [Tester Name],

Thank you for joining the alpha testing for Conset PDF!

Installation:
1. Download the installer from [LINK]
2. Run the installer
3. Launch "Conset PDF" from your Start Menu/Applications

Testing Guide:
Please read the Alpha Testing Guide included in the download.
It explains how to test the app and report bugs.

Key Points:
- This is ALPHA software - expect bugs and rough edges
- Your feedback is invaluable for improving the app
- ALWAYS export logs when reporting a bug (Settings → Debug Logging)
- Don't use on critical files without backups

Reporting Bugs:
Email: [YOUR EMAIL]
Subject: [Bug] Brief description
Attach: Exported log file from the app

Questions?
Feel free to reach out anytime. I'm here to help!

Testing Period:
[START DATE] - [END DATE]

Thanks again for your help!

[Your Name]
```

## Common Issues Checklist

Be prepared to help testers with:

- [ ] "Where do I find the logs?" → Settings → Debug Logging
- [ ] "App won't start" → Check system requirements, antivirus
- [ ] "How do I report this bug?" → Use Export Logs button, send file
- [ ] "Can I use this for real work?" → Not recommended for alpha
- [ ] "When will this be fixed?" → Triage and provide estimate

## Success Metrics

Track these during alpha:

- [ ] Number of testers actively using the app
- [ ] Number of bugs reported
- [ ] Number of bugs fixed
- [ ] Response time to bug reports
- [ ] Tester satisfaction/feedback

## Next Steps

After successful alpha testing:

1. Fix critical and high-priority bugs
2. Update based on tester feedback
3. Move to beta with broader testing
4. Prepare for production release

---

**Ready for Alpha Testing?**

Once all items are checked:
✅ Build the installer
✅ Prepare welcome email
✅ Send to testers
✅ Provide support
✅ Iterate based on feedback

Good luck with your alpha testing! 🚀

# macOS listing fixture

These are prerequisites for the planned installation and unattended chapters, which are still
outline entries. Run only on macOS with `scripts/test-operations-macos.sh`. This validates a
launchd property list without loading a service, and inspects an ad hoc signature on a scratch
copy of the pinned binary.
Signing that copy makes this work on both Intel and Apple Silicon; it does not claim notarization
or a Developer ID signature. The signature's paths and hashes vary, so its details are omitted.

<!-- transcript: file dev.ikigai.fixture.plist -->
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>dev.ikigai.fixture</string>
  <key>ProgramArguments</key>
  <array><string>/usr/local/bin/ikigai</string><string>--version</string></array>
  <key>RunAtLoad</key><true/>
</dict>
</plist>
```

<!-- transcript: run; exit 0 -->
```console
$ plutil -lint dev.ikigai.fixture.plist
dev.ikigai.fixture.plist: OK
```

<!-- transcript: run; exit 0 -->
```console
$ cp "$(command -v ikigai)" ./ikigai
$ codesign --force --sign - ./ikigai 2>/dev/null
$ codesign -dv ./ikigai
Executable=…
…
```

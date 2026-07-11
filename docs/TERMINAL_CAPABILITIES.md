# Terminal Capabilities

Shellow provides the same terminal engine on iOS and Android. Unless noted
otherwise, the capabilities below are implemented by the shared Rust core and
exposed through native SwiftUI and Jetpack Compose interfaces.

## Compatibility summary

| Area | iOS | Android | Notes |
| --- | --- | --- | --- |
| Password authentication | Supported | Supported | Credentials can be stored securely on device. |
| OpenSSH private keys | Supported | Supported | Passphrase-protected keys are supported. |
| Host-key verification and pinning | Supported | Supported | Includes trust-on-first-use workflows. |
| Interactive PTY sessions | Supported | Supported | Backed by `russh`. |
| Terminal emulation | Supported | Supported | Shared persistent `libghostty-vt` state. |
| GPU terminal rendering | Metal | Vulkan | Shared `wgpu` renderer attached to a native surface. |
| Scrollback and jump to bottom | Supported | Supported | Auto-follow pauses while reading older output. |
| Selection, copy, paste, and search | Supported | Supported | Multi-line paste requires confirmation. |
| Direct software keyboard input | Supported | Supported | No visible staging input field is required. |
| Hardware keyboard input | Supported | Supported | Includes navigation and control keys. |
| Mouse reporting | Supported | Supported | Terminal selection yields to mouse-reporting modes. |
| OSC title, bell, and OSC 52 | Supported | Supported | Remote clipboard writes require confirmation. |
| Transcript export | Supported | Supported | Exports visible terminal text as UTF-8. |
| tmux, GNU screen, and Zellij | Supported | Supported | Named persistent sessions and common controls. |
| Host capability detection | Supported | Supported | Detects available persistent-session backends. |

## Input

Shellow sends text, Enter, Escape, Tab, Backspace, arrow keys, Home, End,
Page Up, Page Down, function keys, and common Ctrl/Alt combinations. Local
readline-style editing provides cursor movement, history recall, reverse search,
and common line-editing shortcuts before a command is submitted.

When an application enables bracketed paste, pasted text is wrapped using the
terminal protocol. Multi-line or otherwise risky paste content is shown for
confirmation before it is sent.

## Terminal behavior

The shared VT engine supports normal and alternate screens, styled cells,
256-color and true-color output, cursor styles, scrolling regions, terminal
resize, mouse modes, title changes, bell events, and OSC 52 clipboard requests.

The renderer performs shaped glyph layout and maintains dirty-row updates so a
terminal change does not require rebuilding the full visible grid. Native
overlays remain responsible for hit testing and accessibility.

## Text operations

Users can select visible rows, copy a selection or the terminal, search visible
terminal state, paste local clipboard content, copy recognized links, clear the
visible terminal, reset terminal state, and export a transcript.

Clipboard content requested by a remote process is never copied silently. The
native application presents the requested size and requires user approval.

## Persistent terminal sessions

Profiles may attach to a named tmux, GNU screen, or Zellij session. Shellow can
detect which backends and required command options are available on a host,
then expose backend-specific create, switch, split, and detach controls.

Capability results are cached with the host profile and can be refreshed
without disturbing the active interactive terminal.

## Current limitations

- Complex-script fallback fonts, color emoji, and bidirectional terminal text
  need broader coverage.
- GPU surface behavior and frame pacing can vary across Android vendors and
  should be validated on physical devices.
- SSH host-key and authentication interoperability depends on the algorithms
  supported by `russh` and the remote server.
- Terminal applications may use escape-sequence extensions that are outside the
  implemented `libghostty-vt` behavior.

Implementation details and ownership boundaries are documented in
[Architecture](ARCHITECTURE.md).

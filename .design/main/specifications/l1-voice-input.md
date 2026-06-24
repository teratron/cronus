# Voice Input

**Version:** 1.0.0
**Status:** Stable
**Layer:** concept

## Overview

The voice input subsystem lets users dictate ideas, prompts, and commands by speaking. Spoken audio is captured, processed through an on-device voice-activity detection and transcription pipeline, and injected into the active input field after user review. No audio or transcript leaves the device.

Voice input supplements text input; it does not replace it. The user explicitly controls when recording starts and stops.

## Related Specifications

- [l2-technology-stack.md](l2-technology-stack.md) — audio processing stack (VAD, ONNX, cpal, 16 kHz pipeline) that this spec builds on
- [l1-security.md](l1-security.md) — data residency guarantee: audio stays on-device
- [l1-navigation-model.md](l1-navigation-model.md) — Chat tab (primary surface for voice input)

## 1. Motivation

Dictating ideas is faster than typing for many users and enables capture of fleeting thoughts without requiring a keyboard. Voice input reduces friction when the user wants to brainstorm with the office orchestrator during activities where typing is inconvenient (commute, whiteboard session, hands-occupied work).

## 2. Constraints & Assumptions

- Transcription runs entirely on-device; no audio or transcript is transmitted to any external service.
- The audio pipeline uses the existing 16 kHz mono capture infrastructure from the technology stack.
- Transcription accuracy depends on the on-device model; the transcript is presented for user review before injection.
- Mobile platforms (iOS, Android) require explicit microphone permission grants per platform policy.
- Voice input does not interpret commands directly — the transcript is plain text injected into the input field; the user (or the agent receiving it) handles intent.

## 3. Core Invariants

- **VI-1 On-device only**: audio data MUST NOT leave the device. Transcription uses a locally stored speech model. No network connection is required for voice input.
- **VI-2 User review before injection**: the transcript is shown to the user for confirmation or editing before being injected into the active input field. Silent auto-injection without review is not permitted.
- **VI-3 Explicit activation**: recording begins only on explicit user gesture (push-to-talk or toggle). The application MUST NOT record continuously or passively without explicit per-session consent.
- **VI-4 Active recording indicator**: a prominent visual indicator (and haptic on mobile) is shown for the full duration of recording. The indicator MUST be visible regardless of which tab is active.
- **VI-5 Cancellation without side effects**: the user may cancel a recording at any point before injection. Cancelled recordings discard all audio and transcript atomically — nothing is written to any log or store.

## 4. Detailed Design

### 4.1 Pipeline

```text
[REFERENCE]
ACTIVATE   → user gesture (hold or tap microphone button)
CAPTURE    → cpal audio stream at 16 kHz mono
VAD        → ONNX voice-activity detection; segments speech / silence
TRANSCRIBE → on-device speech-to-text model converts segments to text
REVIEW     → overlay displays transcript; user confirms, edits, or cancels
INJECT     → confirmed text inserted at cursor position in active input field
```

### 4.2 Activation Modes

| Mode | Gesture | Suited for |
| --- | --- | --- |
| Push-to-talk | Hold button; release to transcribe | Short phrases, single sentences |
| Toggle | Tap to start; tap again to stop and transcribe | Longer dictation, multi-sentence |

The active mode is a user preference in Global Settings → Appearance.

### 4.3 Review Overlay

After recording stops, an overlay presents:
- Editable transcript text (with word-level confidence indicators where the model provides them)
- **Confirm** action — inject the (optionally edited) text at the cursor
- **Cancel** action — discard everything (VI-5)

The user may edit the transcript freely before confirming; editing does not re-run transcription.

### 4.4 On-Device Model

The speech model is a quantized model stored in the application's immutable binary directory (per `l2-filesystem-layout.md`). Model selection and update lifecycle: <!-- TBD: define whether the model is bundle-only or user-downloadable from a model catalog -->.

## 5. Implementation Notes

1. The VAD component from the technology stack handles silence detection and auto-stops recording after a configurable silence gap (default: 1.5 s).
2. On mobile, request microphone permission on first use via the platform's standard permission API; never at application launch.
3. The review overlay is a system-level layer (not tab-specific), so it remains visible if the user switches tabs while reviewing.

## 6. Drawbacks & Alternatives

**Alternative: cloud transcription** — higher accuracy via an external STT API. Rejected: violates VI-1 (on-device only) and the client security principle.

**Alternative: direct injection without review** — inject transcript immediately without confirmation. Rejected: violates VI-2; transcription errors would silently corrupt prompts.

## Canonical References

| Alias | Path | Purpose |
| --- | --- | --- |
| `[AUDIO-STACK]` | `.design/main/specifications/l2-technology-stack.md` | 16 kHz pipeline, VAD/ONNX, cpal — audio capture foundation |
| `[FS-LAYOUT]` | `.design/main/specifications/l2-filesystem-layout.md` | Speech model storage location |
| `[SECURITY]` | `.design/main/specifications/l1-security.md` | On-device residency requirement (VI-1) |

## Document History

| Version | Date | Author | Notes |
| --- | --- | --- | --- |
| 1.0.0 | 2026-06-24 | Core Team | Initial spec — VI-1…VI-5, pipeline stages, activation modes, review overlay |

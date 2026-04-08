## Task

#73 Phase 3 discussion - Global chords and always-on key capture

## Status

Not approved for implementation.

This remains a policy and safety decision.

## Why It Is Split Out

The desktop rewrite itself is implementable without this feature.

Global chords are different because they require an always-on global input listener while the tray app is running. That is not just a UI detail. It changes the app's safety posture.

## Current Technical Option

Use `rdev::listen` with a rolling `VecDeque<char>` buffer.

Properties:

- always active while the app process is running
- cannot be paused once started
- can be cleared on window-open transitions
- retains only a short in-memory sequence
- is never written to disk by design

## Unresolved Issue

Even with a tiny in-memory buffer, this is still global key capture.

For this project, that matters because:

- the product context is clinical documentation
- the project guidance says to stop before security-sensitive logic
- the original discussion explicitly asked for this interpretation to be verified rather than assumed

## Two Options

1. Implement in-process global chords with `rdev`.

Pros:

- lowest-latency in-app behavior
- one binary controls the full interaction
- direct section targeting is straightforward

Cons:

- always-on key capture remains a policy risk
- harder to explain cleanly to colleagues
- could make future distribution or trust review harder

2. Defer system-wide chords and use tray plus explicit hotkeys only, or explore espanso as the chord source later.

Pros:

- removes the main policy risk
- simpler implementation and explanation
- still delivers most of the desktop workflow value

Cons:

- loses one-step global chord launch behavior
- splits responsibility if espanso is used later

## Recommendation

Do not implement this yet.

Ship Phases 1 and 2 first. Then decide whether:

- the feature is worth the trust and policy cost
- espanso or another external tool is a better fit than an always-on listener inside Scribblenot

## If Later Approved

A future implementation plan should define:

- exact user-facing disclosure text
- whether the feature is default-off
- config schema for `chord_map`
- how the app distinguishes safe pre-open chord capture from in-window note typing
- how the feature is tested and documented for colleagues

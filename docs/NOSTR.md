# Nostr integration (Epic 5)

## Role

**Discovery and invites only.** Game ticks stay on WebSocket / loopback (Epic 2).

## Plan

1. Ephemeral or user nsec for signing invite events.
2. Custom tags: room code, WS endpoint, game kind, stake suggestion.
3. Configurable relay list (`~/.config/bet/nostr.toml`).
4. CLI: `bet nostr announce` / `bet nostr listen`.

## Non-goals

- Using relays as authoritative game state (latency + policy).
- Real-money zaps in v1 (virtual points only).

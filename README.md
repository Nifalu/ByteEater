# ByteEater

> Because checking the cafeteria menu shouldn't require standing up.

A blazingly fast* CLI tool that fetches your SV Group CH weekly lunch menu so you can pre-judge Wednesday's mystery casserole from the comfort of your terminal.

**\*blazingly fast by Rust standards, which means it shells out to `curl` three times per invocation.**

## Installation

```bash
cargo build --release
```

Requires `curl` on your system. Yes, a Rust program that needs `curl`. We contain multitudes.

## Usage

```bash
# What's for lunch today?
byteeater indulge today

# Already dreading tomorrow?
byteeater indulge tomorrow

# Relive yesterday's disappointment
byteeater indulge yesterday

# Plan your entire week of mediocre dining
byteeater indulge thisweek

# Peek into next week's culinary adventures
byteeater indulge nextweek

# Time travel to a specific date
byteeater indulge 14-03-2025
# Colons work too, if you're feeling fancy
byteeater indulge 14:03:2025

# 3 days from now (for the planners)
byteeater indulge t+3

# Week 42 (the answer to lunch, the universe, and everything)
byteeater indulge w42
```

## Output

Returns JSON because real developers eat their meals in structured data format.

## How It Works

1. Politely asks Firebase for a guest pass
2. Trades that pass for a VIP token
3. Uses the VIP token to access... a lunch menu
4. Strips away the Firestore formatting cruft
5. Presents you with the cold, hard truth about today's soup

## Tech Stack

- **Rust** - for when you want zero-cost abstractions with your zero-cost lunch reviews
- **curl** - because `reqwest` felt like too much commitment
- **serde_json** - turning Firebase's deeply nested nightmares into readable JSON since 2024
- **chrono** - so we know which day to be disappointed about
- **anyhow** - because `.unwrap()` everywhere felt too honest

## License

Unlicensed. Like the chef, probably.

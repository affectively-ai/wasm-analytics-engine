# @affectively/wasm-analytics-engine

`@affectively/wasm-analytics-engine` is a Rust/WebAssembly module for event processing, aggregation, and funnel analysis.

The fair brag is that it stays focused. This package gives you a small analytics core that can run through a WASM entrypoint without pulling in a larger analytics stack.

## What It Helps You Do

- process raw event streams
- compute aggregate metrics
- calculate simple funnel progressions

## Installation

```bash
npm install @affectively/wasm-analytics-engine
```

## Quick Start

```ts
import init, {
  process_events,
  compute_funnel,
  aggregate_metrics,
} from '@affectively/wasm-analytics-engine';

await init();

const processed = process_events(rawEvents);
const funnel = compute_funnel(events, ['signup', 'verify', 'purchase']);
const metrics = aggregate_metrics(events, ['pageviews', 'sessions']);
```

## Why This README Is Grounded

This package does not need to be more than it is. The strongest fair brag is that it already gives you a compact analytics-focused WASM module.

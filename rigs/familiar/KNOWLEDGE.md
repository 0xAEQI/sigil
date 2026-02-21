# Operational Knowledge

Hard-won learnings from production. This file is part of the Familiar's identity — loaded on every session.

## Active State

### Handover (2026-02-13)
- PMS starting_equity fix: saved but NOT deployed. Needs build, commit, purge snapshots, restart.
- FNO encoder training: stalled at 0 steps/sec, LTC head training working fine (30 steps/sec, 734 promotions)
- OMS→PMS refactor plan exists but not started

### Trading Performance
- System is pre-fee positive (+$4.13K) but fees ($16.26K) eat the edge
- Entry fill rate: 37%, SL loss rate: 99.4%, real leverage: 0.26x
- EMS leverage=3.0 is dead code — PMS allocator handles all sizing
- All horizon exits with trailing stops → classified as TrailingStop (exit_reason=4), not Horizon (2)

## AlgoStaking Architecture

### Service Pipeline
```
Data Ingestion → Aggregation → Persistence → Feature → Prediction → Signal → PMS → OMS → EMS
```
- 12 Rust microservices, ZeroMQ pub/sub, FlatBuffers serialization
- API gateway (Axum + sqlx), PMS uses deadpool_postgres (tokio-postgres)
- CLI: `algostaking start|stop|restart|status|health|logs dev|prod [service]`

### Prediction Pipeline
- FNO provides base prediction, LTC refines with `tanh × scale` delta
- Untrained LTC → delta ≈ 0 → FNO passthrough (additive delta pattern)
- Retracement: signed ratio of magnitude (opposite sign), temperature-scaled horizon
- Workers are OS threads (crossbeam channels), not tokio

### Benchmark System
- SYSTEM_BENCHMARK fund, dynamic subscriptions created by PMS signal handler
- BenchmarkAllocator capacity: DYNAMIC = strategies_discovered × budget_per_strategy
- Each new strategy injects $1K via add_capital() into account AND fund StatsTracker
- 479 subscriptions auto-created across 9 venues

### Staking Business Logic
- DB: stake_amount, profit_cap, realized_profit, is_capped, free_stake_balance, is_active
- API: stake, staking-summary, account/balance, pause/start
- PMS gate checks + realized_profit tracking
- Frontend: progress bar, stake action, pause/start toggle, profile stats

## Infrastructure

### PostgreSQL Tuning
- Config: `/etc/postgresql/16/main/conf.d/01-algostaking-tuning.conf`
- shared_buffers: 8GB, effective_cache_size: 96GB
- NVMe: random_page_cost=1.1, effective_io_concurrency=200
- Parallel workers: max 8, per-gather 4
- max_locks_per_transaction: 512 (TimescaleDB chunks)

### TimescaleDB
- v2.25.0, Apache 2 (no native compression without Timescale Cloud)
- Retention: `/etc/algostaking/retention-policy.sh` daily at 4am
- `intents` table DROPPED (obsolete, 30K corrupt chunks from year 294241)

### AlertManager
- 19 alert rules, 6 groups in `/etc/prometheus/rules/algostaking.yml`
- Groups: service_health, pipeline_health, latency, infrastructure, database, zmq_health
- Receiver: default (empty webhook — needs Slack/email config)

### Email
- Postfix send-only SMTP (loopback-only)
- @algostaking.com fails (no incoming MX), external works
- Needs SPF record: `v=spf1 ip4:5.9.83.245 -all`

## RiftDecks

### CN Card API
- Official LoL TCG API: `POST lol-api.playloltcg.com/xcx/card/searchCardCraftWeb` (710 cards, no auth)
- CDN images: `cdn.playloltcg.com` (public PNGs, convert to WebP)
- Set mapping: TCGPlayer SFG = CN SFD (Spiritforged)
- TCGPlayer puts `*` in signature card numbers — don't double-transform
- Build script: `node tools/build-cn-index.mjs [--download]`

## Completed Work
- EMS tick starvation fix, PMS cost-basis filter, OMS dedup fix
- Prediction OOM (worker cap at 500), Kelly criterion allocation
- Auto-fund on registration, system benchmark fund
- Frontend trading table wiring, staking business logic
- Metrics & Grafana overhaul (all 3 trading dashboards rewritten)
- Infrastructure hardening: PostgreSQL tuning, TimescaleDB retention, AlertManager, email verification
- Benchmark pipeline: dynamic allocator, equity model, dynamic fund
- FNO+Signal prediction pipeline fixes
- ZMQ tokio starvation fix

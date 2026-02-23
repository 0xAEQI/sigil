# Pulse Checks

Every 30 minutes, verify:

## 1. Service Health
```bash
algostaking health dev
```
All 12 services should respond. If any are down, check logs and attempt restart.

## 2. Domain Status
Call `rig_status` — check all domains for:
- Crashed spirits (spirits_bonded should be 0 normally)
- Stalled quests (open_quests > 0 with 0 spirits_working for extended time)

## 3. Pending Work
Call `all_ready` — anything unblocked and unassigned across all domains? If so, either:
- Auto-assign to appropriate domain spirit
- Escalate to Emperor if it requires human decision

## 4. Whispers
Call `whisper_read` — check for escalations, crash reports, or spirit requests from scouts.

## 5. Infrastructure
Quick checks (via shell if available):
- Disk: `df -h /` — warn if >80%
- Memory: `free -h` — warn if >90%
- Database: `sudo -u postgres psql -c "SELECT count(*) FROM pg_stat_activity WHERE datname LIKE 'algostaking%';"` — warn if >50 connections
- Prometheus targets: any down?

## 6. Raids
Any cross-domain raids stalled or overdue? Check and report.

## Self-Healing

If any domain is unhealthy:
1. Check spirit logs for the domain
2. Attempt respawn (create a new quest to restart the failed task)
3. If that fails, escalate to Emperor with diagnosis

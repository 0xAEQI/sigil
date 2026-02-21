# Heartbeat Checks

Every 30 minutes, verify:

## 1. Service Health
```bash
algostaking health dev
```
All 12 services should respond. If any are down, check logs and attempt restart.

## 2. Rig Status
Call `rig_status` — check all rigs for:
- Crashed workers (workers_hooked should be 0 normally)
- Stalled beads (open_beads > 0 with 0 workers_working for extended time)

## 3. Pending Work
Call `all_ready` — anything unblocked and unassigned across all rigs? If so, either:
- Auto-assign to appropriate rig worker
- Escalate to Emperor if it requires human decision

## 4. Mail
Call `mail_read` — check for escalations, crash reports, or worker requests from witnesses.

## 5. Infrastructure
Quick checks (via shell if available):
- Disk: `df -h /` — warn if >80%
- Memory: `free -h` — warn if >90%
- Database: `sudo -u postgres psql -c "SELECT count(*) FROM pg_stat_activity WHERE datname LIKE 'algostaking%';"` — warn if >50 connections
- Prometheus targets: any down?

## 6. Convoys
Any cross-rig convoys stalled or overdue? Check and report.

## Self-Healing

If any rig is unhealthy:
1. Check worker logs for the rig
2. Attempt respawn (create a new bead to restart the failed task)
3. If that fails, escalate to Emperor with diagnosis

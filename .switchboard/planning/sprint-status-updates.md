# Sprint Status Updates Required

> This file documents updates needed to `.switchboard/state/sprint-status.yaml`
> Created by: Solution Architect
> Date: 2026-03-02

## Summary

The following new epics and stories should be added to the sprint-status.yaml:

### New Epics (Epic 04-07)

| Epic | Title | Priority | Stories | Points |
|------|-------|----------|---------|--------|
| epic-04 | Discord Gateway - Phase 1: Basic Gateway | 1 | 8 | 22 |
| epic-05 | Discord Gateway - Phase 2: Channel Routing | 2 | 5 | 11 |
| epic-06 | Discord Gateway - Phase 3: Multi-Project | 3 | 6 | 14 |
| epic-07 | Discord Gateway - Phase 4: CLI Integration | 4 | 5 | 10 |

**Total: 4 new epics, 24 new stories, 57 points**

## Epic 04 Stories (Phase 1)

```yaml
- id: "epic-04"
  title: "Discord Gateway - Phase 1: Basic Gateway with Single Project"
  priority: 1
  depends_on: []
  status: "not-started"
  stories:
    - id: "4.1"
      title: "Create gateway module structure"
      points: 1
      type: "infrastructure"
      risk: "low"
      depends_on: []
      status: "not-started"
    - id: "4.2"
      title: "Implement gateway configuration loading"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["4.1"]
      status: "not-started"
    - id: "4.3"
      title: "Create HTTP server with health check endpoint"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["4.2"]
      status: "not-started"
    - id: "4.4"
      title: "Implement WebSocket server for project connections"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["4.3"]
      status: "not-started"
    - id: "4.5"
      title: "Define message protocol types"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["4.1"]
      status: "not-started"
    - id: "4.6"
      title: "Implement basic registration protocol"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["4.4", "4.5"]
      status: "not-started"
    - id: "4.7"
      title: "Wire up Discord Gateway connection"
      points: 5
      type: "feature"
      risk: "high"
      depends_on: ["4.2", "4.6"]
      status: "not-started"
    - id: "4.8"
      title: "Add CLI command switchboard gateway up"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["4.7"]
      status: "not-started"
```

## Epic 05 Stories (Phase 2)

```yaml
- id: "epic-05"
  title: "Discord Gateway - Phase 2: Channel Routing with Config File"
  priority: 2
  depends_on: ["epic-04"]
  status: "not-started"
  stories:
    - id: "5.1"
      title: "Implement ChannelRegistry"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["4.1"]
      status: "not-started"
    - id: "5.2"
      title: "Support channel mapping in config"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["4.2"]
      status: "not-started"
    - id: "5.3"
      title: "Route messages by channel_id"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["5.1", "5.2", "4.7"]
      status: "not-started"
    - id: "5.4"
      title: "Support runtime channel subscribe/unsubscribe"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["5.1"]
      status: "not-started"
    - id: "5.5"
      title: "Add configuration validation"
      points: 1
      type: "feature"
      risk: "low"
      depends_on: ["5.2"]
      status: "not-started"
```

## Epic 06 Stories (Phase 3)

```yaml
- id: "epic-06"
  title: "Discord Gateway - Phase 3: Multi-Project Support & Reconnection"
  priority: 3
  depends_on: ["epic-05"]
  status: "not-started"
  stories:
    - id: "6.1"
      title: "Implement project connection management"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["4.6"]
      status: "not-started"
    - id: "6.2"
      title: "Add heartbeat protocol"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["6.1"]
      status: "not-started"
    - id: "6.3"
      title: "Implement reconnection logic"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["6.2"]
      status: "not-started"
    - id: "6.4"
      title: "Handle project disconnections gracefully"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["6.1"]
      status: "not-started"
    - id: "6.5"
      title: "Implement fan-out message delivery"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["5.3"]
      status: "not-started"
    - id: "6.6"
      title: "Implement Discord rate limit handling"
      points: 2
      type: "feature"
      risk: "medium"
      depends_on: ["4.7"]
      status: "not-started"
```

## Epic 07 Stories (Phase 4)

```yaml
- id: "epic-07"
  title: "Discord Gateway - Phase 4: CLI Integration & Monitoring"
  priority: 4
  depends_on: ["epic-06"]
  status: "not-started"
  stories:
    - id: "7.1"
      title: "Implement switchboard gateway status"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["6.1"]
      status: "not-started"
    - id: "7.2"
      title: "Implement switchboard gateway down"
      points: 2
      type: "feature"
      risk: "low"
      depends_on: ["4.8"]
      status: "not-started"
    - id: "7.3"
      title: "Add PID file management"
      points: 1
      type: "infrastructure"
      risk: "low"
      depends_on: ["4.8"]
      status: "not-started"
    - id: "7.4"
      title: "Add proper logging"
      points: 2
      type: "infrastructure"
      risk: "low"
      depends_on: ["4.1"]
      status: "not-started"
    - id: "7.5"
      title: "Create gateway client library"
      points: 3
      type: "feature"
      risk: "medium"
      depends_on: ["4.5", "4.6", "6.2"]
      status: "not-started"
```

## Action Required

The Sprint Planner or operator should merge these epics into `.switchboard/state/sprint-status.yaml`.

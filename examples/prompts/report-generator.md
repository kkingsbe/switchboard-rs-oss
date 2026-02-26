# Weekly Report Generator

## Task
Generate a comprehensive weekly report of the project's activity based on git commits, issue tracking, and any available documentation.

## Scope
- Review all commits from the past 7 days
- Analyze code changes and their impact
- Summarize key developments
- Identify any issues, blockers, or risks
- List completed features and enhancements
- Track ongoing work items

## Context
- Project: A Rust-based CLI tool for managing kilocode agents
- Workspace contains: source code, tests, documentation, and configuration files
- Development follows a sprint-based workflow with multiple contributors
- Quality standards: high code coverage, comprehensive testing, and detailed documentation

## Output Format
Create a markdown file named `reports/weekly-report-YYYY-MM-DD.md` with the following structure:

```markdown
# Weekly Report: YYYY-MM-DD to YYYY-MM-DD

## Executive Summary
(Conise overview - under 200 words)

## Metrics
- Commits: N
- Files Changed: N
- Lines Added: N
- Lines Deleted: N
- Issues Closed: N
- Issues Opened: N

## Completed Features
- Feature 1: Description and impact
- Feature 2: Description and impact

## Bug Fixes
- Bug 1: Description and resolution
- Bug 2: Description and resolution

## Enhancements & Improvements
- Enhancement 1: Description
- Enhancement 2: Description

## Known Issues & Blockers
- Issue 1: Description and severity (High/Medium/Low)
- Issue 2: Description and severity

## Next Week Priorities
- Priority 1: Description
- Priority 2: Description

## Team Activity
- Contributor 1: Summary of contributions
- Contributor 2: Summary of contributions
```

## Constraints & Preferences
- **Format**: Markdown with consistent heading hierarchy
- **Tone**: Professional, objective, and concise
- **Length**: Keep each section focused and scannable
- **Links**: Include references to relevant commits or issues where appropriate
- **Triage**: Prioritize high-impact changes over minor formatting fixes

## Notes
- Focus on changes that impact functionality, performance, or user experience
- Group related changes under appropriate sections
- Use bullet points for readability in all lists
- Ensure dates follow ISO 8601 format (YYYY-MM-DD)
- If no activity in a particular category, note "No items to report"

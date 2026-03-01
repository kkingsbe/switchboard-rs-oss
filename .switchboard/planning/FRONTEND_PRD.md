# Switchboard Frontend — Product Requirements Document

**Version:** 0.1.0  
**Author:** Kyle  
**Date:** February 27, 2026  
**Status:** Draft  

---

## 1. Overview

The Switchboard Frontend is a Next.js web application that provides a graphical interface for managing Switchboard configurations. It allows users to open a "Switchboard Directory" — any directory containing a `switchboard.toml` file — and provides visual tools for editing configurations, managing prompt files, and monitoring agent schedules. The app runs locally and communicates with the local file system through API routes.

### 1.1 Core Value Proposition

- **Visual Configuration Editor**: A user-friendly editor for `switchboard.toml` with syntax highlighting, validation, and auto-completion
- **Agent Visualization**: Visual representation of all configured agents, their schedules, and relationships
- **Prompt Management**: Browse, preview, create, and edit prompt files with live markdown preview
- **Schedule Visualization**: Calendar or timeline view showing when each agent runs

### 1.2 Scope Definition

**In Scope:**
- Opening and managing a Switchboard Directory
- Visual editor for `switchboard.toml`
- Agent configuration visualization
- Schedule/calendar view of agent runs
- Prompt file browser with markdown preview
- Prompt file creation and editing

**Out of Scope:**
- Running agents (delegated to CLI)
- Docker container management
- Real-time agent execution monitoring
- Discord integration UI
- Metrics visualization (use CLI)
- Remote/hybrid deployment management

---

## 2. User Flow

```
1. Launch Switchboard Frontend
2. Click "Open Directory" or drag-drop a folder
3. App validates that switchboard.toml exists
4. On success: Load configuration and display dashboard
5. User can:
   - Edit switchboard.toml in the visual editor
   - View agent cards with schedule info
   - Browse prompt files in file explorer
   - Preview prompts with markdown rendering
   - Create/edit prompt files
6. Save changes → Write to switchboard.toml
```

---

## 3. UI/UX Specification

### 3.1 Window Model

- Single-page application with sidebar navigation
- Modal dialogs for confirmations, new file creation, settings
- Responsive layout with minimum width of 1024px

### 3.2 Layout Structure

```
┌─────────────────────────────────────────────────────────────┐
│  Header Bar (Title + Directory Path + Actions)              │
├──────────┬──────────────────────────────────────────────────┤
│          │                                                  │
│  Sidebar │              Main Content Area                  │
│  (Nav)   │                                                  │
│          │  - Dashboard (Agent Overview)                     │
│  - Home  │  - Config Editor                                 │
│  - Agents│  - Schedule View                                 │
│  - Prompts│  - Prompt File Browser/Editor                   │
│  - Editor│                                                  │
│          │                                                  │
├──────────┴──────────────────────────────────────────────────┤
│  Status Bar (Validation Status + Last Saved)                │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Visual Design

The application uses a dark theme with clean, modern aesthetics. Specific styling details (colors, typography sizes, spacing values) are to be determined during implementation and should follow modern UI best practices for developer tools.

### 3.4 Components

**Sidebar Navigation**
- Collapsible sidebar with navigation items
- Nav items with icons and labels
- Active and hover states for navigation items

**Agent Card**
- Displays agent name, schedule (human-readable), prompt preview, status indicator
- Actions: Edit, Delete, View Schedule
- Visual states for default, hover, and selected

**Schedule Timeline**
- Horizontal scrollable timeline (24h or 7d view)
- Agent rows with execution blocks
- Color-coded by agent with hover details

**Prompt File List**
- Tree view of prompt directory
- File icons based on type
- Context menu for file operations

**Config Editor**
- Code editor with syntax highlighting for TOML
- Auto-completion for known fields
- Inline validation errors
- Optional split view with preview

**Modal Dialogs**
- Centered overlay dialogs
- Header, content, action buttons structure

---

## 4. Functional Specification

### 4.1 Directory Management

**F1: Open Switchboard Directory**
- User clicks "Open Directory" button or uses Ctrl+O
- Native file dialog opens for directory selection
- App validates directory contains `switchboard.toml`
- If valid: Load configuration, update UI
- If invalid: Show error message "No switchboard.toml found in this directory"

**F2: Recent Directories**
- Store last 10 opened directories
- Display in welcome screen or dropdown
- One-click to reopen

**F3: Directory Change Detection**
- Watch for changes to `switchboard.toml` using file system watcher
- Prompt user to reload if external changes detected
- Auto-reload option in settings

### 4.2 Configuration Editor

**F4: TOML Editor**
- Syntax-highlighted editor for `switchboard.toml`
- Support for all TOML features (tables, arrays, inline tables)
- Line numbers and code folding
- Search and replace (Ctrl+F, Ctrl+H)

**F5: Visual Form Editor**
- Alternative to raw TOML editing
- Form fields for each configuration section:
  - Settings: image_name, image_tag, log_dir, timezone, overlap_mode
  - Agent: name, schedule, prompt/prompt_file, readonly, timeout, env, skills
  - Discord: enabled, token_env, channel_id, llm.*, conversation.*
- Real-time validation against TOML schema
- "Edit as TOML" button to switch to raw editor

**F6: Validation**
- Real-time TOML syntax validation
- Schema validation (required fields, types, allowed values)
- Custom validation rules:
  - Agent names must be unique
  - Schedule must be valid cron expression
  - Timeout must be positive duration
  - Prompt and prompt_file are mutually exclusive
- Validation errors shown inline with red indicators
- Error panel listing all issues with click-to-navigate

**F7: Auto-save and Backup**
- Auto-save after 2 seconds of inactivity
- Create `.switchboard.toml.backup` before each save
- Undo/Redo support (Ctrl+Z, Ctrl+Y)

### 4.3 Agent Visualization

**F8: Agent Dashboard**
- Grid of agent cards showing all configured agents
- Summary statistics:
  - Total agents count
  - Agents running schedule (next 24h)
  - Agents with errors
- Quick filters: All, Active, Read-only

**F9: Agent Detail View**
- Full agent configuration display
- Edit agent properties
- View execution history (if logs available)
- Delete agent (with confirmation)

**F10: Agent Creation**
- "Add Agent" button opens creation wizard
- Steps:
  1. Name (required, unique)
  2. Schedule (cron expression builder with presets)
  3. Prompt type (inline or file)
  4. Additional options (timeout, readonly, env, skills)
- Preview before saving

### 4.4 Schedule Visualization

**F11: Schedule Timeline View**
- Horizontal timeline showing next 24 hours or 7 days
- Each agent represented as a row
- Execution blocks positioned according to schedule
- Click on block shows agent details

**F12: Cron Expression Helper**
- Visual cron builder with dropdowns (minute, hour, day, month, weekday)
- Common presets: "Every hour", "Every day at midnight", "Every Monday", etc.
- Human-readable description of schedule
- Live preview of next 5 execution times

**F13: Schedule Validation**
- Validate cron expressions in real-time
- Show warning for schedules that run too frequently (< 1 minute)
- Display next run times for each agent

### 4.5 Prompt File Management

**F14: Prompt Browser**
- File tree showing all prompt files in the configured prompt directory
- Default prompts directory: `.switchboard/prompts/` (relative to switchboard.toml)
- Auto-detect prompt file path from agent configuration
- Filter by: All, Recently Modified, Unused (not referenced by any agent)

**F15: Prompt Preview**
- Split pane: File list | Markdown preview
- Render markdown with syntax highlighting for code blocks
- Show metadata: File size, last modified, word count
- Show which agents reference this prompt

**F16: Prompt Editor**
- Monaco Editor with Markdown syntax highlighting
- Live preview pane (toggleable)
- Toolbar: Bold, Italic, Headers, Lists, Code blocks, Links
- Template insertion for common prompt structures

**F17: Prompt Creation**
- "New Prompt" button in prompt browser
- Create in default prompts directory or specify path
- Choose template: Blank, Agent Task, Report Generator, Custom
- Default filename: `new-prompt.md`

**F18: Prompt Validation**
- Warn if prompt file referenced by agent doesn't exist
- Show unused prompts (not referenced by any agent)
- Character count and token estimate (optional)

### 4.6 Settings

**F19: Application Settings**
- Theme: Dark (default), Light (future)
- Editor font size: 12-20px
- Auto-save interval: 0 (disabled), 1s, 2s, 5s
- Default prompts directory path
- Show hidden files in browser

---

## 5. Data Flow & Architecture

### 5.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Next.js Application                       │
├─────────────────────────────────────────────────────────────┤
│  Client (Browser)          │       Server (API Routes)       │
│  ┌───────────────────┐    │    ┌─────────────────────────┐│
│  │ React Components  │    │    │ FileSystem API           ││
│  │ - Dashboard       │    │    │ - /api/config/read       ││
│  │ - ConfigEditor    │    │    │ - /api/config/write      ││
│  │ - Timeline        │    │    │ - /api/prompts/*         ││
│  │ - PromptBrowser   │    │    │ - /api/directory/open    ││
│  └─────────┬─────────┘    │    └───────────┬─────────────┘│
│            │              │                │               │
│  ┌─────────▼─────────┐    │    ┌───────────▼─────────────┐ │
│  │ State Management  │◄───┼───►│ Node.js fs module       │ │
│  │ (Zustand)         │    │    │ (local file access)     │ │
│  └───────────────────┘    │    └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 API Routes

| Route | Method | Description |
|-------|--------|-------------|
| `/api/directory/open` | POST | Validate and open a switchboard directory |
| `/api/config/read` | GET | Read and parse switchboard.toml |
| `/api/config/write` | POST | Write configuration to switchboard.toml |
| `/api/prompts/list` | GET | List all prompt files in prompts directory |
| `/api/prompts/read` | GET | Read a specific prompt file |
| `/api/prompts/write` | POST | Write/create a prompt file |
| `/api/prompts/delete` | DELETE | Delete a prompt file |

### 5.3 Client-Side State

| Store | Responsibility | State |
|-------|----------------|-------|
| `directoryStore` | Directory state | path, isValid, errors |
| `configStore` | Configuration state | agents, settings, validation |
| `promptStore` | Prompt files state | files, activeFile, content |

---

## 6. Edge Cases

| Scenario | Handling |
|----------|----------|
| No switchboard.toml in directory | Show error toast, stay on welcome screen |
| switchboard.toml is malformed | Show parsing error with line number, allow editing raw |
| Prompt file referenced but missing | Show warning icon on agent card |
| Circular prompt references | Not applicable (prompts are flat files) |
| Directory deleted while app open | Detect via watcher, show error, prompt to open new |
| Very large switchboard.toml (>1MB) | Warn user, still attempt to load |
| Unicode in file paths | Handle properly via Node.js native APIs |
| Concurrent external edits | Prompt user to reload or overwrite |
| No prompts directory exists | Offer to create default `.switchboard/prompts/` |

---

## 7. Acceptance Criteria

### 7.1 Directory Management

- [ ] User can open a directory via button click
- [ ] User can open a directory via drag-and-drop
- [ ] App validates switchboard.toml exists
- [ ] App shows error message for invalid directories
- [ ] Recent directories list shows last 10 opened

### 7.2 Configuration Editor

- [ ] TOML syntax highlighting works correctly
- [ ] Auto-completion suggests known fields
- [ ] Validation errors shown inline with red indicators
- [ ] User can switch between form editor and raw TOML
- [ ] Changes auto-save after 2 seconds of inactivity

### 7.3 Agent Visualization

- [ ] All agents display as cards on dashboard
- [ ] Agent cards show name, schedule, prompt preview
- [ ] User can add new agent via wizard
- [ ] User can edit existing agent
- [ ] User can delete agent with confirmation

### 7.4 Schedule View

- [ ] Timeline shows next 24 hours of agent runs
- [ ] Timeline shows next 7 days of agent runs
- [ ] Cron expression builder produces valid expressions
- [ ] Human-readable schedule description is accurate

### 7.5 Prompt Management

- [ ] Prompt file tree displays correctly
- [ ] Markdown preview renders properly
- [ ] User can create new prompt file
- [ ] User can edit prompt file with live preview
- [ ] User can delete prompt file with confirmation

### 7.6 Visual Checkpoints

1. **Welcome Screen**: Clean, centered "Open Directory" button with recent directories below
2. **Dashboard**: Grid of agent cards with consistent spacing and clear typography
3. **Editor**: Split view with editor on left, form on right (when using form mode)
4. **Timeline**: Horizontal scrollable area with color-coded agent blocks
5. **Prompt Browser**: Tree view on left, preview on right with proper markdown styling
6. **Error States**: Red indicators visible, error messages helpful and actionable

---

## 8. Technical Stack

| Layer | Technology | Rationale |
|-------|------------|------------|
| Framework | Next.js 14+ (App Router) | Local web app with API routes for file access |
| UI Library | React 18+ | Component-based, mature ecosystem |
| State Management | Zustand | Simple, TypeScript-friendly, minimal boilerplate |
| Editor | Monaco Editor | VS Code's editor, excellent TOML support |
| Markdown | react-markdown + remark-gfm | Full GitHub-flavored markdown support |
| Styling | Tailwind CSS | Rapid development, consistent design system |
| Icons | Lucide React | Clean, consistent icon set |
| Date/Time | date-fns + cronstrue | Date manipulation and cron to human-readable |
| File Access | Next.js API Routes + Node.js fs | Server-side file operations |
| Validation | zod | Schema validation for config and forms |

---

## 9. File Structure

```
switchboard-frontend/
├── src/
│   ├── app/                         # Next.js App Router
│   │   ├── layout.tsx               # Root layout
│   │   ├── page.tsx                 # Home/welcome page
│   │   ├── dashboard/               # Dashboard view
│   │   │   └── page.tsx
│   │   ├── editor/                  # Config editor view
│   │   │   └── page.tsx
│   │   ├── schedule/                # Schedule timeline view
│   │   │   └── page.tsx
│   │   ├── prompts/                 # Prompt browser/editor
│   │   │   └── page.tsx
│   │   └── api/                    # API routes
│   │       ├── directory/
│   │       │   └── route.ts
│   │       ├── config/
│   │       │   └── route.ts
│   │       └── prompts/
│   │           ├── route.ts
│   │           └── [path]/
│   │               └── route.ts
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── StatusBar.tsx
│   │   ├── dashboard/
│   │   │   ├── AgentCard.tsx
│   │   │   └── AgentGrid.tsx
│   │   ├── editor/
│   │   │   ├── ConfigEditor.tsx
│   │   │   └── AgentForm.tsx
│   │   ├── schedule/
│   │   │   ├── Timeline.tsx
│   │   │   └── CronBuilder.tsx
│   │   └── prompts/
│   │       ├── PromptTree.tsx
│   │       ├── PromptPreview.tsx
│   │       └── PromptEditor.tsx
│   ├── lib/
│   │   ├── config.ts                # Config parsing utilities
│   │   ├── validation.ts            # Validation logic
│   │   └── cron.ts                  # Cron parsing utilities
│   ├── stores/
│   │   ├── directoryStore.ts
│   │   ├── configStore.ts
│   │   └── promptStore.ts
│   └── styles/
│       └── globals.css
├── public/
├── package.json
├── next.config.js
├── tailwind.config.js
└── tsconfig.json
```

---

## 10. Future Considerations (Out of Scope for v0.1)

- **Light Theme**: Dark theme only for initial release
- **Metrics Dashboard**: Use CLI for metrics viewing
- **Real-time Execution Monitor**: Stream agent logs in real-time
- **Multi-instance Support**: Manage multiple switchboard directories
- **Plugin System**: Extend with custom agent types
- **Cloud Sync**: Sync configuration across devices
- **Team Sharing**: Share configurations via URL

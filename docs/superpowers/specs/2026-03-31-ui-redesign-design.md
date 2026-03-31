# Ren API Proxy - UI Redesign Design

**Date:** 2026-03-31
**Status:** Approved

## Overview

Migrate the Ren API Proxy frontend from vanilla JavaScript to Vite + React + Ant Design, with a new clean & minimal visual design.

## Design Direction

**Style:** Modern Split — clean sections with subtle gray backgrounds, pill-style status indicator, dark accent button.

## UI/UX Specification

### Layout Structure

- Single page application
- Two stacked sections (cards) with rounded corners (12px)
- Provider Settings section on top
- Authentication section below
- Footer area with status indicator and action buttons

### Visual Design

**Color Palette:**
- Background: `#f5f5f5`
- Section background: `#fafafa`
- Input background: `#ffffff`
- Primary button: `#1a1a1a` (dark)
- Secondary button: `#ffffff` with `#e0e0e0` border
- Status pill (stopped): `#f0f0f0` background, `#666` text
- Status pill (running): `#f6ffed` background, `#52c41a` text (green)
- Text labels: `#999` (uppercase, small)
- Text inputs: `#333`

**Typography:**
- Font family: `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`
- Section headers: 12px, uppercase, letter-spacing 0.5px, `#999`
- Inputs: 14px
- Buttons: 14px, weight 500

**Spacing:**
- Section padding: 20px
- Section gap: 20px
- Input padding: 12px
- Button padding: 14px

**Visual Effects:**
- Input shadow: `0 1px 2px rgba(0,0,0,0.05)`
- Card hover: subtle lift with `translateY(-4px)` and enhanced shadow (for future enhancements)
- Button transitions: 0.2s ease

### Components

1. **Section Card**
   - Background: `#fafafa`
   - Border radius: 12px
   - Contains: header label + form fields

2. **Provider Settings Section**
   - Header: "PROVIDER SETTINGS"
   - Fields: Provider dropdown, Port input (horizontal row)

3. **Authentication Section**
   - Header: "AUTHENTICATION"
   - Fields: API Key input (password field)

4. **Status Pill**
   - Stopped: gray background, gray dot
   - Running: green tint background, green dot
   - Contains: colored dot + text

5. **Action Buttons**
   - Start: dark background (#1a1a1a), white text, full width
   - Stop: white background, dark text, border

6. **Connection URL Display**
   - Appears when proxy is running
   - Copy-to-clipboard functionality
   - Shows URL in monospace font

## Functionality Specification

### Core Features

1. **Load Configuration**
   - On app start, load saved config from Tauri backend
   - Populate provider, port, upstream URL fields

2. **Start Proxy Service**
   - Validate API key is not empty
   - Save configuration to backend
   - Start proxy server
   - Display connection URL when started
   - Update status to "Running"

3. **Stop Proxy Service**
   - Stop proxy server
   - Hide connection URL
   - Update status to "Stopped"

4. **Get Proxy Status**
   - Check if proxy is currently running
   - Update UI accordingly on load

### User Interactions

- Click "Start" → validates → saves config → starts proxy → shows URL
- Click "Stop" → stops proxy → hides URL
- Changing provider updates upstream URL preset (optional enhancement)

### Data Handling

- Use Tauri invoke commands to communicate with Rust backend
- State management via React useState/useEffect

## Technical Implementation

### Stack

- **Build Tool:** Vite
- **Framework:** React 18+
- **UI Library:** Ant Design 5.x
- **Tauri Integration:** @tauri-apps/api

### File Structure

```
src/
├── main.jsx          # React entry point
├── App.jsx           # Main app component
├── App.css           # Custom styles (minimal overrides)
├── components/
│   └── ProxyForm.jsx # Main form component
└── index.html        # HTML template (minimal)
```

### Tauri Commands

- `get_config` — load saved configuration
- `save_proxy_config` — save provider, API key, port, upstream URL
- `start_proxy` — start the proxy server
- `stop_proxy` — stop the proxy server
- `get_proxy_status` — check if proxy is running
- `get_local_ip` — get local IP address

## Acceptance Criteria

1. ✅ App loads and displays the new UI
2. ✅ Configuration loads from backend on startup
3. ✅ Start button validates and starts proxy
4. ✅ Stop button stops the proxy
5. ✅ Status indicator reflects actual proxy state
6. ✅ Connection URL displays when running
7. ✅ Clean, minimal visual design matches spec
8. ✅ Responsive and works in desktop window
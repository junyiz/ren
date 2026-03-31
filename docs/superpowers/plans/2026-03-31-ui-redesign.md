# UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate frontend from vanilla JS to Vite + React + Ant Design with modern split UI design.

**Architecture:** Replace static HTML/JS with Vite-powered React app. Use Ant Design components for UI, integrate with existing Tauri commands. Keep Rust backend unchanged.

**Tech Stack:** Vite, React 18, Ant Design 5.x, @tauri-apps/api

---

### Task 1: Setup Vite + React + Ant Design

**Files:**
- Modify: `package.json` — add Vite, React, Ant Design dependencies
- Create: `vite.config.js` — Vite configuration for Tauri

- [ ] **Step 1: Update package.json with dependencies**

Run: `cat > /Users/junyi/codes/ren/package.json << 'EOF'
{
  "name": "ren-desktop",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "antd": "^5.22.0",
    "@tauri-apps/api": "^2.2.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@vitejs/plugin-react": "^4.3.4",
    "vite": "^6.0.7"
  },
  "packageManager": "pnpm@10.33.0"
}
EOF`

- [ ] **Step 2: Create vite.config.js**

Run: `cat > /Users/junyi/codes/ren/vite.config.js << 'EOF'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    outDir: 'dist',
    target: 'esnext',
    minify: 'esbuild',
  },
})
EOF`

- [ ] **Step 3: Commit**

Run:
```bash
git add package.json vite.config.js
git commit -m "feat: add Vite, React, and Ant Design dependencies"
```

---

### Task 2: Create React App Structure

**Files:**
- Create: `index.html` — new HTML entry point
- Create: `src/main.jsx` — React entry point
- Create: `src/App.jsx` — Main app component
- Create: `src/App.css` — Custom styles
- Delete: `src/index.html` (old), `src/main.js` (old), `src/styles.css`

- [ ] **Step 1: Create index.html**

Run: `cat > /Users/junyi/codes/ren/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Ren API Proxy</title>
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/src/main.jsx"></script>
</body>
</html>
EOF`

- [ ] **Step 2: Create src/main.jsx**

Run: `cat > /Users/junyi/codes/ren/src/main.jsx << 'EOF'
import React from 'react'
import ReactDOM from 'react-dom/client'
import { ConfigProvider } from 'antd'
import App from './App'
import './App.css'

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: '#1a1a1a',
          borderRadius: 8,
          fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        },
      }}
    >
      <App />
    </ConfigProvider>
  </React.StrictMode>
)
EOF`

- [ ] **Step 3: Create src/App.jsx**

Run: `cat > /Users/junyi/codes/ren/src/App.jsx << 'EOF'
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Form, Select, InputNumber, Button, Card, Space, Typography, message } from 'antd'

const { Title, Text } = Typography

const PROVIDER_OPTIONS = [
  { value: 'openai', label: 'OpenAI', upstream: 'https://api.openai.com' },
  { value: 'anthropic', label: 'Anthropic', upstream: 'https://api.anthropic.com' },
  { value: 'ollama', label: 'Ollama', upstream: 'http://localhost:11434' },
]

function App() {
  const [isRunning, setIsRunning] = useState(false)
  const [proxyUrl, setProxyUrl] = useState('')
  const [loading, setLoading] = useState(false)
  const [form] = Form.useForm()

  useEffect(() => {
    loadConfig()
    checkStatus()
  }, [])

  const loadConfig = async () => {
    try {
      const config = await invoke('get_config')
      form.setFieldsValue({
        provider: config.provider,
        port: config.port,
        upstreamUrl: config.upstream_url,
      })
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  const checkStatus = async () => {
    try {
      const running = await invoke('get_proxy_status')
      setIsRunning(running)
    } catch (e) {
      console.error('Failed to check status:', e)
    }
  }

  const handleProviderChange = (value) => {
    const provider = PROVIDER_OPTIONS.find(p => p.value === value)
    if (provider) {
      form.setFieldValue('upstreamUrl', provider.upstream)
    }
  }

  const handleStart = async () => {
    try {
      const values = await form.validateFields()
      if (!values.apiKey) {
        message.error('Please enter your API key')
        return
      }

      setLoading(true)
      await invoke('save_proxy_config', {
        provider: values.provider,
        apiKey: values.apiKey,
        port: values.port,
        upstreamUrl: values.upstreamUrl,
      })

      const url = await invoke('start_proxy')
      setProxyUrl(url)
      setIsRunning(true)
      message.success('Proxy started')
    } catch (e) {
      message.error('Failed to start: ' + e)
    } finally {
      setLoading(false)
    }
  }

  const handleStop = async () => {
    try {
      setLoading(true)
      await invoke('stop_proxy')
      setIsRunning(false)
      setProxyUrl('')
      message.success('Proxy stopped')
    } catch (e) {
      message.error('Failed to stop: ' + e)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="app-container">
      <div className="app-header">
        <Title level={3} style={{ margin: 0 }}>Ren API Proxy</Title>
        <Text type="secondary">Share your LLM API within LAN safely</Text>
      </div>

      <Card className="section-card">
        <div className="section-label">Provider Settings</div>
        <Form form={form} layout="vertical">
          <Space size={16} style={{ width: '100%' }}>
            <Form.Item name="provider" style={{ flex: 1 }}>
              <Select
                options={PROVIDER_OPTIONS}
                onChange={handleProviderChange}
                placeholder="Select provider"
              />
            </Form.Item>
            <Form.Item name="port" initialValue={8080} style={{ width: 120 }}>
              <InputNumber min={1024} max={65535} style={{ width: '100%' }} />
            </Form.Item>
          </Space>
          <Form.Item name="upstreamUrl" label="Upstream URL">
            <InputNumber />
          </Form.Item>
        </Form>
      </Card>

      <Card className="section-card">
        <div className="section-label">Authentication</div>
        <Form form={form} layout="vertical">
          <Form.Item
            name="apiKey"
            rules={[{ required: true, message: 'Please enter your API key' }]}
          >
            <Input.Password
              placeholder="sk-..."
              size="large"
            />
          </Form.Item>
        </Form>
      </Card>

      <div className="status-bar">
        <div className={`status-pill ${isRunning ? 'active' : ''}`}>
          <span className="status-dot" />
          {isRunning ? 'Running' : 'Stopped'}
        </div>
        <div className="button-group">
          <Button
            type="primary"
            size="large"
            onClick={handleStart}
            disabled={isRunning}
            loading={loading}
            className="btn-start"
          >
            Start
          </Button>
          <Button
            size="large"
            onClick={handleStop}
            disabled={!isRunning}
            loading={loading}
            className="btn-stop"
          >
            Stop
          </Button>
        </div>
      </div>

      {proxyUrl && (
        <Card className="url-card">
          <div className="section-label">Share this URL</div>
          <div className="proxy-url">{proxyUrl}</div>
        </Card>
      )}
    </div>
  )
}

export default App
EOF`

- [ ] **Step 4: Create src/App.css**

Run: `cat > /Users/junyi/codes/ren/src/App.css << 'EOF'
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #f5f5f5;
  min-height: 100vh;
}

.app-container {
  max-width: 600px;
  margin: 0 auto;
  padding: 32px 24px;
}

.app-header {
  text-align: center;
  margin-bottom: 32px;
}

.app-header .ant-typography {
  margin-bottom: 4px;
}

.section-card {
  background: #fafafa;
  border: none;
  border-radius: 12px;
  margin-bottom: 16px;
}

.section-card .ant-card-body {
  padding: 20px;
}

.section-label {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #999;
  margin-bottom: 16px;
}

.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: white;
  border-radius: 12px;
  margin-top: 8px;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 13px;
  background: #f0f0f0;
  color: #666;
}

.status-pill.active {
  background: #f6ffed;
  color: #52c41a;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #999;
}

.status-pill.active .status-dot {
  background: #52c41a;
}

.button-group {
  display: flex;
  gap: 12px;
}

.btn-start {
  background: #1a1a1a;
  border-color: #1a1a1a;
}

.btn-start:hover {
  background: #333;
  border-color: #333;
}

.btn-stop {
  color: #1a1a1a;
  border-color: #e0e0e0;
}

.btn-stop:hover {
  color: #333;
  border-color: #ccc;
}

.url-card {
  margin-top: 16px;
  background: #f5f5ff;
  border: none;
  border-radius: 12px;
}

.url-card .ant-card-body {
  padding: 20px;
  text-align: center;
}

.proxy-url {
  font-family: monospace;
  color: #667eea;
  font-size: 14px;
  padding: 12px;
  background: white;
  border-radius: 8px;
  margin-top: 8px;
}
EOF`

- [ ] **Step 5: Delete old frontend files**

Run:
```bash
rm -f /Users/junyi/codes/ren/src/index.html /Users/junyi/codes/ren/src/main.js /Users/junyi/codes/ren/src/styles.css /Users/junyi/codes/ren/src/assets/javascript.svg /Users/junyi/codes/ren/src/assets/tauri.svg
```

- [ ] **Step 6: Commit**

Run:
```bash
git add -A
git commit -m "feat: migrate to Vite + React + Ant Design UI"
```

---

### Task 3: Update Tauri Config for Vite Build

**Files:**
- Modify: `src-tauri/tauri.conf.json` — update frontendDist to use Vite output

- [ ] **Step 1: Update tauri.conf.json**

Run: `cat > /Users/junyi/codes/ren/src-tauri/tauri.conf.json << 'EOF'
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Ren API Proxy",
  "version": "0.1.0",
  "identifier": "im.ren.api-proxy",
  "build": {
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Ren API Proxy",
        "width": 640,
        "height": 700,
        "resizable": true,
        "center": true,
        "devtools": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
EOF`

- [ ] **Step 2: Commit**

Run:
```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: update tauri config for Vite build"
```

---

### Task 4: Install Dependencies and Test Build

**Files:**
- Test: Verify npm install works
- Test: Verify Vite build works

- [ ] **Step 1: Install dependencies**

Run: `cd /Users/junyi/codes/ren && pnpm install`

- [ ] **Step 2: Test Vite build**

Run: `cd /Users/junyi/codes/ren && pnpm build`

Expected: Build completes without errors, creates `dist/` folder

- [ ] **Step 3: Commit**

Run:
```bash
git add package-lock.json pnpm-lock.yaml
git commit -m "chore: add lock files"
```

---

### Task 5: Verify Dev Server Works

**Files:**
- Test: Verify Vite dev server starts

- [ ] **Step 1: Test dev server starts**

Run: `cd /Users/junyi/codes/ren && timeout 10 pnpm dev || true`

Expected: Dev server starts on port 1420, no errors

- [ ] **Step 2: Commit**

Run:
```bash
git status
git add -A
git commit -m "chore: ensure all changes committed"
```

---

### Task 6: Build and Test Tauri App

**Files:**
- Test: Build Tauri app
- Test: Verify app runs

- [ ] **Step 1: Build Tauri app**

Run: `cd /Users/junyi/codes/ren && pnpm tauri build`

Expected: Build completes successfully

- [ ] **Step 2: Commit**

Run:
```bash
git add -A
git commit -m "feat: complete UI redesign with Vite + React + Ant Design"
```

---

## Acceptance Criteria Verification

- [ ] package.json has Vite, React, Ant Design dependencies
- [ ] vite.config.js created with Tauri-compatible settings
- [ ] index.html is minimal, loads React from /src/main.jsx
- [ ] src/main.jsx renders App with Ant Design ConfigProvider
- [ ] src/App.jsx implements Modern Split design with sections, status pill, buttons
- [ ] src/App.css has custom styles matching spec
- [ ] Old frontend files removed
- [ ] tauri.conf.json updated with dist as frontendDist
- [ ] `pnpm build` creates dist/ folder
- [ ] `pnpm tauri build` succeeds
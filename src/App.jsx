import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Form, Select, InputNumber, Input, Button, Card, Space, Typography, message } from 'antd'

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
      const errorMsg = e?.message || e?.toString() || String(e)
      message.error('Failed to start: ' + errorMsg)
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
            <Input />
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
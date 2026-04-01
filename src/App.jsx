import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Form,
  Select,
  InputNumber,
  Input,
  Button,
  Card,
  Typography,
  message,
  Switch,
  Modal,
  Collapse,
} from "antd";
import { CopyOutlined, CheckOutlined } from "@ant-design/icons";

const { Title, Text } = Typography;

function UrlList({ urls }) {
  return (
    <Card className="url-card">
      {urls.map(({ url, label }) => (
        <div key={label} style={{ marginBottom: 12 }}>
          <div className="section-label">{label}</div>
          <Input
            value={url}
            readOnly
            addonAfter={
              <Button
                type="text"
                icon={<CopyOutlined />}
                onClick={async () => {
                  try {
                    await navigator.clipboard.writeText(url);
                    message.success("Copied to clipboard");
                  } catch (e) {
                    message.error("Failed to copy");
                  }
                }}
                style={{ height: 28 }}
              />
            }
            style={{ fontFamily: "monospace" }}
          />
        </div>
      ))}
    </Card>
  );
}

const PROVIDER_OPTIONS = [
  { value: "openai", label: "OpenAI", upstream: "https://api.openai.com" },
  {
    value: "anthropic",
    label: "Anthropic",
    upstream: "https://api.anthropic.com",
  },
  { value: "ollama", label: "Ollama", upstream: "http://localhost:11434" },
];

const RELAY_OPTIONS = [
  { value: "ren.im", label: "ren.im" },
  { value: "tunelo.net", label: "tunelo.net" },
];

function App() {
  const [isRunning, setIsRunning] = useState(false);
  const [proxyUrl, setProxyUrl] = useState("");
  const [publicUrl, setPublicUrl] = useState("");
  const [publicAccess, setPublicAccess] = useState(false);
  const [relay, setRelay] = useState("ren.im");
  const [loading, setLoading] = useState(false);
  const [configExpanded, setConfigExpanded] = useState(true);
  const [form] = Form.useForm();

  useEffect(() => {
    loadConfig();
    checkStatus();
  }, []);

  const loadConfig = async () => {
    try {
      const config = await invoke("get_config");
      form.setFieldsValue({
        provider: config.provider,
        port: config.port,
        upstreamUrl: config.upstream_url,
      });
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  };

  const checkStatus = async () => {
    try {
      const running = await invoke("get_proxy_status");
      setIsRunning(running);
      if (running) {
        // Get local IP for local URL
        const localIp = await invoke("get_local_ip");
        const config = await invoke("get_config");
        setProxyUrl(`http://${localIp}:${config.port}/v1`);
      }
      // Check tunnel status
      const tunnelUrl = await invoke("get_tunnel_status");
      if (tunnelUrl) {
        setPublicUrl(tunnelUrl + "/v1");
        setPublicAccess(true);
      }
    } catch (e) {
      console.error("Failed to check status:", e);
    }
  };

  const handleProviderChange = (value) => {
    const provider = PROVIDER_OPTIONS.find((p) => p.value === value);
    if (provider) {
      form.setFieldValue("upstreamUrl", provider.upstream);
    }
  };

  const handleStart = async () => {
    const values = await form.validateFields();
    if (!values.apiKey) {
      message.error("Please enter your API key");
      return;
    }
    try {
      setLoading(true);
      await invoke("save_proxy_config", {
        provider: values.provider,
        apiKey: values.apiKey,
        port: values.port,
        upstreamUrl: values.upstreamUrl,
      });

      const url = await invoke("start_proxy");
      setProxyUrl(url);

      // Start tunnel if public access is enabled
      if (publicAccess) {
        try {
          const tunnelUrl = await invoke("start_tunnel", {
            port: values.port,
            relay: relay,
          });
          setPublicUrl(tunnelUrl + "/v1");
        } catch (e) {
          Modal.warning({
            title: "Tunnel Failed",
            content: "Tunnel failed to start: " + e,
          });
          setPublicAccess(false);
        }
      }

      setIsRunning(true);
      setConfigExpanded(false);
      message.success("Proxy started");
    } catch (e) {
      message.error("Failed to start: " + e);
    } finally {
      setLoading(false);
    }
  };

  const handleStop = async () => {
    try {
      setLoading(true);
      await invoke("stop_proxy");
      setIsRunning(false);
      setConfigExpanded(true);
      setProxyUrl("");
      setPublicUrl("");
      message.success("Proxy stopped");
    } catch (e) {
      message.error("Failed to stop: " + e);
    } finally {
      setLoading(false);
    }
  };

  const handlePublicAccessChange = (checked) => {
    setPublicAccess(checked);
  };

  return (
    <div className="app-container">
      <div className="app-header">
        <Title level={3} style={{ margin: 0 }}>
          Ren API Proxy
        </Title>
        <Text type="secondary">
          Share your LLM API over LAN or internet — key stays local
        </Text>
      </div>

      <Collapse
        activeKey={configExpanded ? ["config"] : []}
        onChange={(keys) => setConfigExpanded(keys.includes("config"))}
        items={[
          {
            key: "config",
            label: "Provider Settings",
            children: (
              <Form form={form}>
                <Form.Item
                  name="provider"
                  label="Provider"
                  rules={[{ required: true, message: "Please select a provider" }]}
                >
                  <Select
                    options={PROVIDER_OPTIONS}
                    onChange={handleProviderChange}
                    placeholder="Select provider"
                    disabled={isRunning}
                  />
                </Form.Item>
                <Form.Item
                  name="upstreamUrl"
                  label="Upstream URL"
                  rules={[
                    { required: true, message: "Please enter the upstream URL" },
                  ]}
                >
                  <Input disabled={isRunning} />
                </Form.Item>
                <Form.Item
                  name="apiKey"
                  label="API Key"
                  rules={[{ required: true, message: "Please enter your API key" }]}
                >
                  <Input.Password
                    placeholder="sk-..."
                    size="large"
                    disabled={isRunning}
                  />
                </Form.Item>
                <div className="section-label">Local Settings</div>
                <Form.Item name="port" label="Port" initialValue={8080}>
                  <InputNumber style={{ width: "100%" }} disabled={isRunning} />
                </Form.Item>
                <div
                  className="section-label"
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                  }}
                >
                  <span>Public Access</span>
                  <Switch
                    checked={publicAccess}
                    onChange={handlePublicAccessChange}
                    disabled={isRunning}
                  />
                </div>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  Enable to create a public tunnel and share over the internet
                </Text>
                {publicAccess && (
                  <Form.Item label="Relay" style={{ marginTop: 12, marginBottom: 0 }}>
                    <Select
                      value={relay}
                      onChange={setRelay}
                      options={RELAY_OPTIONS}
                      disabled={isRunning}
                    />
                  </Form.Item>
                )}
              </Form>
            ),
          },
        ]}
      />

      <div className="status-bar">
        <div className={`status-pill ${isRunning ? "active" : ""}`}>
          <span className="status-dot" />
          {isRunning ? "Running" : "Stopped"}
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

      {isRunning && (
        <UrlList
          urls={[
            { url: proxyUrl, label: "Local URL (LAN)" },
            ...(publicUrl
              ? [{ url: publicUrl, label: "Public URL (Internet)" }]
              : []),
          ]}
        />
      )}
    </div>
  );
}

export default App;

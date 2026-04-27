<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

<p align="center">
  <strong>English</strong> | <a href="docs/README.zh-CN.md">中文</a>
</p>

# Ren AI Proxy

An AI quota proxy tool that supports sharing over local area networks and public networks. It allows secure access to LLM resources without the need to distribute API keys, effectively eliminating the risk of key leakage.

<p align="center">
  <img src="docs/images/ren.png" width="480" alt="Desktop application for sharing LLM API within LAN or over the internet without exposing keys" />
</p>


## Features

- Share your OpenAI/Anthropic/Ollama API with others on your network
- Your API key never leaves your device
- Simple one-click setup
- Works with any OpenAI-compatible client
- **Public access**: Create a public tunnel to share your proxy over the internet — your API key stays on your machine

## Installation

### Build from source

```bash
cd ren
npm install
npm run tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

### Pre-built binaries

Download from the [releases page](https://github.com/junyiz/ren/releases).

> First launch note: macOS will block unsigned apps downloaded from the internet. After dragging Ren AI Proxy to Applications, open Terminal and run:
> 
> ```bash
> xattr -cr /Applications/Ren\ AI\ Proxy.app/
> ```

## Usage

1. Download and install Ren AI Proxy
2. Enter your API key in the text field
3. Select your provider (OpenAI, Anthropic, or Ollama)
4. Click "Start Service"
5. Share the displayed URL with others on your network

### Public Access (Internet)

Enable "Public Access" to create a public tunnel via tunelo relay. Your API key stays on your local machine and is never shared with the relay service.

## For Users Connecting to Your Proxy

Set your client's `base_url` to the URL shown in the app:

```python
from openai import AsyncOpenAI

client = AsyncOpenAI(
    base_url="http://192.168.1.x:8090/v1",  # Use the URL from the app
    api_key="anything"  # Can be anything, won't be used
)
```

```bash
curl -s https://stupendous-division-5473.ren.im/v1/chat/completions \
    -H "Content-Type: application/json" \
    -d '{
      "model": "kimi-k2.5",
      "messages": [{"role": "user", "content": "hi"}]
    }'
```

## Security

Your API key is encrypted locally and never leaves your device. The proxy only forwards requests between the client and the LLM provider.

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Acknowledgments
[tunelo](https://tunelo.net) · [plano](https://github.com/katanemo/plano)

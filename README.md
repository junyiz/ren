# Ren API Proxy

Desktop application for sharing LLM API keys within LAN without exposing keys.

## Features

- Share your OpenAI/Anthropic/Ollama API keys with others on your network
- Your API key never leaves your device
- Simple one-click setup
- Works with any OpenAI-compatible client

## Installation

### Build from source

```bash
cd ren-desktop
npm install
npm run tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

### Pre-built binaries

Download from the [releases page](https://github.com/junyiz/ren/releases).

## Usage

1. Download and install Ren API Proxy
2. Enter your API key in the text field
3. Select your provider (OpenAI, Anthropic, or Ollama)
4. Click "Start Service"
5. Share the displayed URL with others on your network

## For Users Connecting to Your Proxy

Set your client's `base_url` to the URL shown in the app:

```python
from openai import AsyncOpenAI

client = AsyncOpenAI(
    base_url="http://192.168.1.x:8080/v1",  # Use the URL from the app
    api_key="anything"  # Can be anything, won't be used
)
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
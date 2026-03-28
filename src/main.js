const { invoke } = window.__TAURI__.core;

let isRunning = false;

async function loadConfig() {
    try {
        const config = await invoke('get_config');
        document.getElementById('provider').value = config.provider;
        document.getElementById('port').value = config.port;
        document.getElementById('upstream-url').value = config.upstream_url;
    } catch (e) {
        console.error('Failed to load config:', e);
    }
}

async function checkStatus() {
    try {
        isRunning = await invoke('get_proxy_status');
        updateUI();
    } catch (e) {
        console.error('Failed to check status:', e);
    }
}

function updateUI() {
    const indicator = document.getElementById('status-indicator');
    const statusText = document.getElementById('status-text');
    const startBtn = document.getElementById('start-btn');
    const stopBtn = document.getElementById('stop-btn');
    const connectionInfo = document.getElementById('connection-info');

    if (isRunning) {
        indicator.classList.add('running');
        statusText.textContent = 'Service: Running';
        startBtn.disabled = true;
        stopBtn.disabled = false;
    } else {
        indicator.classList.remove('running');
        statusText.textContent = 'Service: Stopped';
        startBtn.disabled = false;
        stopBtn.disabled = true;
        connectionInfo.style.display = 'none';
    }
}

async function startProxy() {
    const provider = document.getElementById('provider').value;
    const apiKey = document.getElementById('api-key').value;
    const port = parseInt(document.getElementById('port').value);
    const upstreamUrl = document.getElementById('upstream-url').value;

    if (!apiKey) {
        alert('Please enter your API key');
        return;
    }

    try {
        // Save config first
        await invoke('save_proxy_config', {
            provider,
            apiKey,
            port,
            upstreamUrl
        });

        // Start proxy
        const proxyUrl = await invoke('start_proxy');
        isRunning = true;

        document.getElementById('proxy-url').textContent = proxyUrl;
        document.getElementById('connection-info').style.display = 'block';

        updateUI();
    } catch (e) {
        alert('Failed to start: ' + e);
    }
}

async function stopProxy() {
    try {
        await invoke('stop_proxy');
        isRunning = false;
        updateUI();
    } catch (e) {
        alert('Failed to stop: ' + e);
    }
}

// Event listeners
document.getElementById('start-btn').addEventListener('click', startProxy);
document.getElementById('stop-btn').addEventListener('click', stopProxy);

// Initialize
loadConfig();
checkStatus();
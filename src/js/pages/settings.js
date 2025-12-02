import { getConfig, setApiKey, setBaseUrl, setModel, setProvider } from '../api.js';
import { showSuccess, showError } from '../components/toast.js';
import { router } from '../router.js';
import { checkForUpdates } from '../updater.js';

// Provider presets with default values
const PROVIDERS = {
  openai: {
    name: 'OpenAI',
    baseUrl: 'https://api.openai.com/v1',
    model: 'gpt-5.1',
    available: true,
    description: 'GPT models via OpenAI API'
  },
  openrouter: {
    name: 'OpenRouter',
    baseUrl: 'https://openrouter.ai/api/v1',
    model: 'anthropic/claude-sonnet-4',
    available: true,
    description: 'Access multiple providers through one API'
  },
  custom: {
    name: 'Custom Endpoint',
    baseUrl: '',
    model: '',
    available: true,
    description: 'Any OpenAI-compatible API endpoint'
  },
  anthropic: {
    name: 'Anthropic',
    baseUrl: 'https://api.anthropic.com/v1',
    model: 'claude-sonnet-4-20250514',
    available: false,
    description: 'Not available - use OpenRouter for Claude models'
  },
  google: {
    name: 'Google AI',
    baseUrl: 'https://generativelanguage.googleapis.com/v1beta',
    model: 'gemini-pro',
    available: false,
    description: 'Not available - only OpenAI-compatible APIs supported'
  }
};

export async function renderSettings() {
  const app = document.getElementById('app');

  // Load current config
  let config = { provider: 'openai', base_url: null, model: null, api_key: null };
  try {
    config = await getConfig();
  } catch (e) {
    // Use defaults
  }

  const currentProvider = config.provider || 'openai';
  const providerInfo = PROVIDERS[currentProvider] || PROVIDERS.openai;

  app.innerHTML = `
    <div class="main-content">
      <div class="settings-container">
        <div class="settings-header">
          <h1 class="settings-logo">Liminal</h1>
          <p class="settings-subtitle">Build Your Custom Learning Experience</p>
        </div>

        <!-- Provider Configuration -->
        <div class="card">
          <h3 class="card-title">AI Provider</h3>
          <p class="card-description">Configure your OpenAI-compatible API provider.</p>

          <div class="form-group" style="margin-top: 16px;">
            <label class="form-label">Provider</label>
            <select id="provider-select" class="input input-lg">
              ${Object.entries(PROVIDERS).map(([key, p]) => `
                <option value="${key}" ${currentProvider === key ? 'selected' : ''}>${p.name}${!p.available ? ' (Coming Soon)' : ''}</option>
              `).join('')}
            </select>
            <p class="form-hint" id="provider-hint">${providerInfo.description}</p>
          </div>

          <div id="provider-unavailable" class="provider-unavailable ${providerInfo.available ? 'hidden' : ''}">
            <p>This provider is not yet available. Only OpenAI-compatible APIs are supported at the moment.</p>
            <p>Try <strong>OpenAI</strong>, <strong>OpenRouter</strong>, or a <strong>Custom Endpoint</strong>.</p>
          </div>

          <div id="config-fields" class="${!providerInfo.available ? 'fields-disabled' : ''}">
            <div class="form-group">
              <label class="form-label">Base URL</label>
              <input
                type="text"
                id="base-url-input"
                class="input input-lg"
                placeholder="https://api.openai.com/v1"
                value="${config.base_url || providerInfo.baseUrl || ''}"
                ${!providerInfo.available ? 'disabled' : ''}
              />
            </div>

            <div class="form-group">
              <label class="form-label">Model</label>
              <input
                type="text"
                id="model-input"
                class="input input-lg"
                placeholder="gpt-5.1"
                value="${config.model || providerInfo.model || ''}"
                ${!providerInfo.available ? 'disabled' : ''}
              />
            </div>

            <div class="form-group">
              <label class="form-label">API Key</label>
              <div class="input-with-action">
                <input
                  type="password"
                  id="api-key-input"
                  class="input input-lg"
                  placeholder="Enter your API key..."
                  value="${config.api_key || ''}"
                  ${!providerInfo.available ? 'disabled' : ''}
                />
                <button type="button" id="clear-key-btn" class="btn btn-secondary btn-icon" title="Clear API key" ${!config.api_key || !providerInfo.available ? 'disabled' : ''}>
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                  </svg>
                </button>
              </div>
              <p class="form-hint">Your API key is stored locally and never shared.</p>
            </div>

            <button id="save-config-btn" class="btn btn-primary" style="margin-top: 16px; width: 100%;" ${!providerInfo.available ? 'disabled' : ''}>
              Save Configuration
            </button>
          </div>
        </div>

        <div style="margin-top: 24px;">
          <button id="back-btn" class="btn btn-secondary btn-lg" style="width: 100%;">Back to Library</button>
        </div>

        <!-- Version Info -->
        <div class="version-info">
          <span id="version-text">Version ${window.__TAURI__?.app?.getVersion ? 'loading...' : 'unknown'}</span>
          <button id="check-updates-btn" class="version-link">Check for updates</button>
        </div>
      </div>
    </div>
  `;

  // Load version
  loadVersion();

  // Event listeners
  setupEventListeners();
}

async function loadVersion() {
  try {
    const { getVersion } = window.__TAURI__.app;
    const version = await getVersion();
    document.getElementById('version-text').textContent = `Version ${version}`;
  } catch (e) {
    console.error('Failed to get version:', e);
    document.getElementById('version-text').textContent = 'Version unknown';
  }
}

function setupEventListeners() {
  // Back button
  document.getElementById('back-btn').addEventListener('click', () => {
    router.navigate('/projects');
  });

  // Check for updates button
  document.getElementById('check-updates-btn').addEventListener('click', async () => {
    const btn = document.getElementById('check-updates-btn');
    btn.textContent = 'Checking...';
    btn.disabled = true;

    try {
      await checkForUpdates(false);
    } finally {
      btn.textContent = 'Check for updates';
      btn.disabled = false;
    }
  });

  // Provider select change
  const providerSelect = document.getElementById('provider-select');
  providerSelect.addEventListener('change', (e) => {
    const provider = e.target.value;
    const preset = PROVIDERS[provider];

    // Update hint
    document.getElementById('provider-hint').textContent = preset.description;

    // Show/hide unavailable message
    const unavailableMsg = document.getElementById('provider-unavailable');
    const configFields = document.getElementById('config-fields');

    if (!preset.available) {
      unavailableMsg.classList.remove('hidden');
      configFields.classList.add('fields-disabled');
    } else {
      unavailableMsg.classList.add('hidden');
      configFields.classList.remove('fields-disabled');
    }

    // Update fields
    const baseUrlInput = document.getElementById('base-url-input');
    const modelInput = document.getElementById('model-input');
    const apiKeyInput = document.getElementById('api-key-input');
    const saveBtn = document.getElementById('save-config-btn');
    const clearBtn = document.getElementById('clear-key-btn');

    baseUrlInput.value = preset.baseUrl;
    modelInput.value = preset.model;

    baseUrlInput.disabled = !preset.available;
    modelInput.disabled = !preset.available;
    apiKeyInput.disabled = !preset.available;
    saveBtn.disabled = !preset.available;
    clearBtn.disabled = !preset.available || !apiKeyInput.value;
  });

  // Clear API key button
  document.getElementById('clear-key-btn').addEventListener('click', async () => {
    const apiKeyInput = document.getElementById('api-key-input');
    const clearBtn = document.getElementById('clear-key-btn');

    try {
      await setApiKey('');
      apiKeyInput.value = '';
      clearBtn.disabled = true;
      showSuccess('API key cleared');
    } catch (e) {
      showError('Failed to clear API key: ' + e);
    }
  });

  // Update clear button state when API key changes
  document.getElementById('api-key-input').addEventListener('input', (e) => {
    document.getElementById('clear-key-btn').disabled = !e.target.value;
  });

  // Save configuration button
  document.getElementById('save-config-btn').addEventListener('click', async () => {
    const provider = document.getElementById('provider-select').value;
    const baseUrl = document.getElementById('base-url-input').value.trim();
    const model = document.getElementById('model-input').value.trim();
    const apiKey = document.getElementById('api-key-input').value.trim();

    // Validation
    if (!apiKey) {
      showError('API key is required');
      return;
    }

    if (!baseUrl) {
      showError('Base URL is required');
      return;
    }

    if (!model) {
      showError('Model is required');
      return;
    }

    try {
      // Save all config values
      await setProvider(provider);
      await setBaseUrl(baseUrl);
      await setModel(model);
      await setApiKey(apiKey);

      showSuccess('Configuration saved!');
    } catch (e) {
      showError('Failed to save: ' + e);
    }
  });
}

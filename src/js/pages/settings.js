import { getApiKey, setApiKey } from '../api.js';
import { showSuccess, showError } from '../components/toast.js';
import { router } from '../router.js';

export async function renderSettings() {
  const app = document.getElementById('app');

  app.innerHTML = `
    <div class="main-content">
      <div class="settings-container">
        <div class="settings-header">
          <h1 class="settings-logo">Liminal</h1>
          <p class="settings-subtitle">Build Your Custom Learning Experience</p>
        </div>

        <div class="card">
          <div class="form-group">
            <label class="form-label">API Key</label>
            <input
              type="password"
              id="api-key-input"
              class="input input-lg"
              placeholder="Enter your API key..."
            />
            <p class="form-hint">
              Your API key is stored locally and never shared.
            </p>
          </div>

          <div style="display: flex; gap: 12px; margin-top: 24px;">
            <button id="save-btn" class="btn btn-primary btn-lg" style="flex: 1;">Save</button>
            <button id="back-btn" class="btn btn-secondary btn-lg">Back</button>
          </div>
        </div>
      </div>
    </div>
  `;

  // Load existing key
  try {
    const existingKey = await getApiKey();
    if (existingKey) {
      document.getElementById('api-key-input').value = existingKey;
    }
  } catch (e) {
    // No key set yet
  }

  // Event listeners
  document.getElementById('save-btn').addEventListener('click', async () => {
    const key = document.getElementById('api-key-input').value.trim();

    if (!key) {
      showError('Please enter an API key');
      return;
    }

    try {
      await setApiKey(key);
      showSuccess('API key saved!');
      router.navigate('/projects');
    } catch (e) {
      showError('Failed to save: ' + e);
    }
  });

  document.getElementById('back-btn').addEventListener('click', () => {
    router.navigate('/projects');
  });
}

import { getApiKey, setApiKey, checkAuthStatus, startSignIn, signOut, fetchApiKeyFromServer } from '../api.js';
import { showSuccess, showError } from '../components/toast.js';
import { router } from '../router.js';

let isSigningIn = false;

export async function renderSettings() {
  const app = document.getElementById('app');

  // Check auth status first
  let authStatus = { isAuthenticated: false, email: null };
  try {
    authStatus = await checkAuthStatus();
  } catch (e) {
    // Not authenticated
  }

  app.innerHTML = `
    <div class="main-content">
      <div class="settings-container">
        <div class="settings-header">
          <h1 class="settings-logo">Liminal</h1>
          <p class="settings-subtitle">Build Your Custom Learning Experience</p>
        </div>

        <div class="card">
          ${authStatus.isAuthenticated ? renderAuthenticatedUI(authStatus) : renderSignInUI()}
        </div>

        <!-- Advanced Settings (collapsed by default) -->
        <details class="advanced-settings">
          <summary class="advanced-settings-toggle">Advanced Settings</summary>
          <div class="card" style="margin-top: 12px;">
            <div class="form-group">
              <label class="form-label">API Key (Manual Override)</label>
              <input
                type="password"
                id="api-key-input"
                class="input input-lg"
                placeholder="Enter your API key..."
              />
              <p class="form-hint">
                Only use this if you have your own API key. Otherwise, sign in above.
              </p>
            </div>
            <button id="save-key-btn" class="btn btn-secondary" style="margin-top: 12px;">Save API Key</button>
          </div>
        </details>

        <div style="margin-top: 24px;">
          <button id="back-btn" class="btn btn-secondary btn-lg" style="width: 100%;">Back to Library</button>
        </div>
      </div>
    </div>
  `;

  // Load existing API key for advanced settings
  try {
    const existingKey = await getApiKey();
    if (existingKey) {
      document.getElementById('api-key-input').value = existingKey;
    }
  } catch (e) {
    // No key set yet
  }

  // Event listeners
  setupEventListeners(authStatus.isAuthenticated);
}

function renderSignInUI() {
  return `
    <div class="auth-section">
      <div class="auth-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
          <circle cx="12" cy="7" r="4"></circle>
        </svg>
      </div>
      <h3 class="auth-title">Sign in to get started</h3>
      <p class="auth-description">
        Sign in with your Liminal account to access AI-powered learning features.
      </p>
      <button id="signin-btn" class="btn btn-primary btn-lg auth-btn">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path>
          <polyline points="10 17 15 12 10 7"></polyline>
          <line x1="15" y1="12" x2="3" y2="12"></line>
        </svg>
        Sign In
      </button>
    </div>
  `;
}

function renderAuthenticatedUI(authStatus) {
  return `
    <div class="auth-section">
      <div class="auth-icon auth-icon-success">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
          <polyline points="22 4 12 14.01 9 11.01"></polyline>
        </svg>
      </div>
      <h3 class="auth-title">You're signed in</h3>
      <p class="auth-email">${authStatus.email || 'Unknown email'}</p>
      <p class="auth-description">
        Your account is connected and ready to use.
      </p>
      <button id="signout-btn" class="btn btn-secondary auth-btn">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
          <polyline points="16 17 21 12 16 7"></polyline>
          <line x1="21" y1="12" x2="9" y2="12"></line>
        </svg>
        Sign Out
      </button>
    </div>
  `;
}

function setupEventListeners(isAuthenticated) {
  // Back button
  document.getElementById('back-btn').addEventListener('click', () => {
    router.navigate('/projects');
  });

  // Save API key button (advanced settings)
  document.getElementById('save-key-btn').addEventListener('click', async () => {
    const key = document.getElementById('api-key-input').value.trim();

    if (!key) {
      showError('Please enter an API key');
      return;
    }

    try {
      await setApiKey(key);
      showSuccess('API key saved!');
    } catch (e) {
      showError('Failed to save: ' + e);
    }
  });

  // Auth-specific buttons
  if (isAuthenticated) {
    document.getElementById('signout-btn').addEventListener('click', handleSignOut);
  } else {
    document.getElementById('signin-btn').addEventListener('click', handleSignIn);
  }
}

async function handleSignIn() {
  if (isSigningIn) return;

  const btn = document.getElementById('signin-btn');
  isSigningIn = true;
  btn.disabled = true;
  btn.innerHTML = `
    <div class="btn-spinner"></div>
    Opening browser...
  `;

  try {
    await startSignIn();
    // Browser will open, and the app will receive a deep link callback
    // The callback is handled in app.js
    btn.innerHTML = `
      <div class="btn-spinner"></div>
      Waiting for authorization...
    `;
  } catch (e) {
    showError('Failed to start sign in: ' + e);
    isSigningIn = false;
    btn.disabled = false;
    btn.innerHTML = `
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path>
        <polyline points="10 17 15 12 10 7"></polyline>
        <line x1="15" y1="12" x2="3" y2="12"></line>
      </svg>
      Sign In
    `;
  }
}

async function handleSignOut() {
  try {
    await signOut();
    showSuccess('Signed out successfully');
    // Re-render the settings page
    renderSettings();
  } catch (e) {
    showError('Failed to sign out: ' + e);
  }
}

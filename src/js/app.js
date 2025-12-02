import { router } from './router.js';
import { renderSettings } from './pages/settings.js';
import { renderProjects } from './pages/projects.js';
import { renderReader } from './pages/reader.js';
import { getApiKey, checkAuthStatus, fetchApiKeyFromServer, handleAuthCallback } from './api.js';
import { initUpdater } from './updater.js';
import { showSuccess, showError } from './components/toast.js';

const { listen } = window.__TAURI__.event;

// Register routes
router.register('/settings', renderSettings);
router.register('/projects', renderProjects);
router.register('/project/:id', renderReader);

// Listen for auth callback events from deep links
async function setupAuthListener() {
  await listen('auth-callback', async (event) => {
    const { accessToken, refreshToken, expiresIn, userId, email, state } = event.payload;

    try {
      // Handle the auth callback
      const authStatus = await handleAuthCallback(
        accessToken,
        refreshToken,
        expiresIn,
        userId,
        email,
        state
      );

      if (authStatus.isAuthenticated) {
        showSuccess('Signed in successfully!');

        // Fetch API key from server
        try {
          await fetchApiKeyFromServer();
          showSuccess('Ready to learn!');
        } catch (e) {
          console.error('Failed to fetch API key:', e);
          showError('Subscription required to access AI features');
        }

        // Navigate to projects
        router.navigate('/projects');
      }
    } catch (e) {
      showError('Sign in failed: ' + e);
      router.navigate('/settings');
    }
  });
}

// Check for API key on startup and route accordingly
async function init() {
  // Set up auth callback listener first
  await setupAuthListener();

  try {
    // First check if user is authenticated
    const authStatus = await checkAuthStatus();

    if (authStatus.isAuthenticated) {
      // User is authenticated, try to get/refresh API key
      try {
        const apiKey = await getApiKey();

        if (!apiKey) {
          // No API key stored, fetch from server
          await fetchApiKeyFromServer();
        }

        // API key is available, start normally
        router.start();
      } catch (e) {
        // Failed to get API key, might need to re-authenticate
        console.error('Failed to get API key:', e);
        router.navigate('/settings');
      }
    } else {
      // Not authenticated, check for legacy API key
      const apiKey = await getApiKey();

      if (apiKey) {
        // Has legacy API key, let them continue
        router.start();
      } else {
        // No auth and no API key, go to settings
        router.navigate('/settings');
      }
    }

    // Initialize auto-updater (checks silently on startup)
    initUpdater();
  } catch (e) {
    // Error checking auth, fall back to API key check
    console.error('Auth check failed:', e);

    try {
      const apiKey = await getApiKey();
      if (apiKey) {
        router.start();
      } else {
        router.navigate('/settings');
      }
    } catch (e2) {
      router.navigate('/settings');
    }
  }
}

// Initialize the app
init();

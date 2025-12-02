import { router } from './router.js';
import { renderSettings } from './pages/settings.js';
import { renderProjects } from './pages/projects.js';
import { renderReader } from './pages/reader.js';
import { getApiKey } from './api.js';
import { initUpdater } from './updater.js';

// Register routes
router.register('/settings', renderSettings);
router.register('/projects', renderProjects);
router.register('/project/:id', renderReader);

// Check for API key on startup and route accordingly
async function init() {
  try {
    const apiKey = await getApiKey();
    if (!apiKey) {
      router.navigate('/settings');
    } else {
      router.start();
    }

    // Initialize auto-updater (checks silently on startup)
    initUpdater();
  } catch (e) {
    // No API key set, go to settings
    router.navigate('/settings');
  }
}

// Initialize the app
init();

import { router } from './router.js';
import { renderSettings } from './pages/settings.js';
import { renderProjects } from './pages/projects.js';
import { renderReader } from './pages/reader.js';
import { initUpdater } from './updater.js';

// Register routes
router.register('/settings', renderSettings);
router.register('/projects', renderProjects);
router.register('/project/:id', renderReader);

// Initialize the app
async function init() {
  // Start the router (will go to /projects by default)
  router.start();

  // Initialize auto-updater (checks silently on startup)
  initUpdater();
}

init();

// Auto-updater module
import { showSuccess, showError } from './components/toast.js';

let updateAvailable = null;

export async function checkForUpdates(silent = false) {
  try {
    const { check } = window.__TAURI__.updater;
    const { getVersion } = window.__TAURI__.app;

    const currentVersion = await getVersion();
    console.log('Current version:', currentVersion);
    console.log('Checking for updates...');

    const update = await check();
    console.log('Update check result:', update);

    if (update) {
      console.log('Update available:', update.version);
      updateAvailable = update;
      if (!silent) {
        showUpdateAvailable(update);
      }
      return update;
    } else {
      console.log('No update available');
      if (!silent) {
        showSuccess('You are on the latest version!');
      }
      return null;
    }
  } catch (e) {
    console.error('Update check failed:', e);
    if (!silent) {
      showError('Failed to check for updates: ' + e.message);
    }
    return null;
  }
}

export async function installUpdate() {
  if (!updateAvailable) {
    showError('No update available');
    return;
  }

  try {
    showSuccess('Downloading update...');

    // Download and install
    await updateAvailable.downloadAndInstall();

    // Relaunch the app
    const { relaunch } = window.__TAURI__.process;
    await relaunch();
  } catch (e) {
    console.error('Update install failed:', e);
    showError('Failed to install update: ' + e);
  }
}

function showUpdateAvailable(update) {
  // Create update notification modal
  const modal = document.createElement('div');
  modal.className = 'update-modal';
  modal.innerHTML = `
    <div class="update-modal-overlay"></div>
    <div class="update-modal-content">
      <h3>Update Available</h3>
      <p>A new version of Liminal is available!</p>
      <p class="update-version">Version ${update.version}</p>
      ${update.body ? `<div class="update-notes">${update.body}</div>` : ''}
      <div class="update-actions">
        <button class="btn btn-secondary" id="update-later">Later</button>
        <button class="btn btn-primary" id="update-now">Update Now</button>
      </div>
    </div>
  `;

  document.body.appendChild(modal);

  // Event listeners
  modal.querySelector('#update-later').addEventListener('click', () => {
    modal.remove();
  });

  modal.querySelector('#update-now').addEventListener('click', async () => {
    modal.querySelector('#update-now').textContent = 'Downloading...';
    modal.querySelector('#update-now').disabled = true;
    modal.querySelector('#update-later').disabled = true;
    await installUpdate();
  });

  modal.querySelector('.update-modal-overlay').addEventListener('click', () => {
    modal.remove();
  });
}

// Check for updates on app start (silent)
export function initUpdater() {
  // Check after a short delay to not block app startup
  setTimeout(() => {
    checkForUpdates(true).then(update => {
      if (update) {
        showUpdateAvailable(update);
      }
    });
  }, 3000);
}

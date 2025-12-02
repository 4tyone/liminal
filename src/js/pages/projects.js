import { listProjects, createProject, deleteProject, generateLearning, importFolder } from '../api.js';
import { showSuccess, showError } from '../components/toast.js';
import { router } from '../router.js';
import { showGenerationLoading, hideLoading } from '../components/loading.js';
import { confirmAction } from '../components/confirm-modal.js';

const { open: openDialog } = window.__TAURI__.dialog;

let isGenerating = false;

export async function renderProjects() {
  const app = document.getElementById('app');

  app.innerHTML = `
    <!-- Top Bar -->
    <header class="topbar">
      <div class="topbar-content">
        <div class="topbar-left">
          <h1 class="topbar-title">Liminal</h1>
        </div>
        <div class="topbar-right">
          <button class="icon-btn" id="settings-btn" aria-label="Settings">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
            </svg>
          </button>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="main-content">
      <div style="max-width: 1100px; margin: 0 auto;">
        <!-- Create New Section -->
        <div class="create-section">
          <div class="create-input-wrapper" id="create-wrapper">
            <textarea
              id="topic-input"
              class="create-input"
              placeholder="What would you like to learn today?"
              rows="1"
            ></textarea>
            <select id="depth-select" class="create-depth">
              <option value="beginner">Brief</option>
              <option value="intermediate" selected>Standard</option>
              <option value="advanced">Comprehensive</option>
            </select>
            <button id="generate-btn" class="create-submit" aria-label="Generate">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="22" y1="2" x2="11" y2="13"></line>
                <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
              </svg>
            </button>
          </div>
          <div id="agent-status" class="agent-status"></div>
          <button id="import-btn" class="import-btn">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="17 8 12 3 7 8"></polyline>
              <line x1="12" y1="3" x2="12" y2="15"></line>
            </svg>
            Import folder
          </button>
        </div>

        <!-- Projects Grid -->
        <div id="projects-grid" class="projects-grid">
          <!-- Projects loaded here -->
        </div>
      </div>
    </main>
  `;

  // Load projects
  await loadProjects();

  // Event listeners
  document.getElementById('settings-btn').addEventListener('click', () => {
    router.navigate('/settings');
  });

  const topicInput = document.getElementById('topic-input');

  // Auto-resize textarea
  topicInput.addEventListener('input', () => {
    topicInput.style.height = 'auto';
    topicInput.style.height = topicInput.scrollHeight + 'px';
  });

  // Enter to submit, Shift+Enter for new line
  topicInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleGenerate();
    }
  });

  // Generate button click
  document.getElementById('generate-btn').addEventListener('click', handleGenerate);

  // Import button click
  document.getElementById('import-btn').addEventListener('click', handleImport);
}

async function loadProjects() {
  const grid = document.getElementById('projects-grid');

  try {
    const projects = await listProjects();

    if (projects.length === 0) {
      grid.innerHTML = `
        <div class="empty-state" style="grid-column: 1 / -1;">
          <p class="empty-state-title">No projects yet</p>
          <p class="empty-state-desc">Create your first learning below!</p>
        </div>
      `;
      return;
    }

    grid.innerHTML = projects.map(project => `
      <div class="project-card" data-project-id="${project.id}">
        <div class="project-card-header">
          <h3 class="project-card-title">${escapeHtml(project.title)}</h3>
          <button class="icon-btn project-card-delete" data-id="${project.id}" aria-label="Delete">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
        <p class="project-card-desc">${escapeHtml(project.description || 'No description')}</p>
        <div class="project-card-meta">
          ${project.pageCount || 0} pages · ${formatDate(project.updatedAt)}
        </div>
      </div>
    `).join('');

    // Click handlers
    grid.querySelectorAll('.project-card').forEach(card => {
      card.addEventListener('click', (e) => {
        if (e.target.closest('.project-card-delete')) return;
        router.navigate(`/project/${card.dataset.projectId}`);
      });
    });

    grid.querySelectorAll('.project-card-delete').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        e.stopPropagation();
        const projectCard = btn.closest('.project-card');
        const projectTitle = projectCard.querySelector('.project-card-title').textContent;

        const confirmed = await confirmAction(
          'Delete Learning',
          `Are you sure you want to delete "${projectTitle}"? This action cannot be undone.`
        );

        if (confirmed) {
          try {
            await deleteProject(btn.dataset.id);
            showSuccess('Learning deleted');
            loadProjects();
          } catch (e) {
            showError('Failed to delete: ' + e);
          }
        }
      });
    });

  } catch (e) {
    grid.innerHTML = `
      <div class="empty-state" style="grid-column: 1 / -1; color: var(--color-error);">
        Failed to load projects: ${e}
      </div>
    `;
  }
}

async function handleGenerate() {
  if (isGenerating) return;

  const topic = document.getElementById('topic-input').value.trim();
  const depth = document.getElementById('depth-select').value;

  if (!topic) {
    showError('Please enter a topic');
    return;
  }

  const wrapper = document.getElementById('create-wrapper');
  const input = document.getElementById('topic-input');
  isGenerating = true;
  wrapper.classList.add('loading');
  input.disabled = true;
  showGenerationLoading();

  try {
    const project = await generateLearning(topic, depth);
    showSuccess('Learning material generated!');
    router.navigate(`/project/${project.id}`);
  } catch (e) {
    showError('Failed to generate: ' + e);
  } finally {
    isGenerating = false;
    wrapper.classList.remove('loading');
    input.disabled = false;
    hideLoading();
  }
}

async function handleImport() {
  try {
    // Open folder picker dialog
    const folderPath = await openDialog({
      directory: true,
      multiple: false,
      title: 'Select folder with markdown files'
    });

    if (!folderPath) return; // User cancelled

    // Show a simple prompt for title and description
    const title = await promptForInput('Learning Title', 'Enter a title for this learning:');
    if (!title) return; // User cancelled

    const description = await promptForInput('Description', 'Enter a brief description (optional):') || '';

    showGenerationLoading();

    const project = await importFolder(folderPath, title, description);
    showSuccess('Folder imported successfully!');
    router.navigate(`/project/${project.id}`);
  } catch (e) {
    showError('Failed to import: ' + e);
  } finally {
    hideLoading();
  }
}

function promptForInput(title, message) {
  return new Promise((resolve) => {
    // Create modal overlay
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    overlay.innerHTML = `
      <div class="modal">
        <h3 class="modal-title">${title}</h3>
        <p class="modal-message">${message}</p>
        <input type="text" class="modal-input" id="prompt-input" autofocus />
        <div class="modal-buttons">
          <button class="modal-btn modal-btn-cancel" id="prompt-cancel">Cancel</button>
          <button class="modal-btn modal-btn-confirm" id="prompt-confirm">OK</button>
        </div>
      </div>
    `;

    document.body.appendChild(overlay);

    const input = overlay.querySelector('#prompt-input');
    const cancelBtn = overlay.querySelector('#prompt-cancel');
    const confirmBtn = overlay.querySelector('#prompt-confirm');

    const cleanup = (value) => {
      overlay.remove();
      resolve(value);
    };

    cancelBtn.addEventListener('click', () => cleanup(null));
    confirmBtn.addEventListener('click', () => cleanup(input.value.trim()));

    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') cleanup(input.value.trim());
      if (e.key === 'Escape') cleanup(null);
    });

    // Focus input after a short delay
    setTimeout(() => input.focus(), 100);
  });
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function formatDate(dateStr) {
  if (!dateStr) return 'Unknown';
  const date = new Date(dateStr);
  return date.toLocaleDateString();
}

import { getProject, getPageContent, expandSelection, exportToPdf } from '../api.js';
import { showError, showSuccess } from '../components/toast.js';
import { initSelectionPopover, cleanupSelectionPopover, hidePopover, hidePopoverLoading } from '../components/selection-popover.js';
import { renderMarkdown } from '../markdown.js';
import { router } from '../router.js';

let currentProject = null;
let currentPageIndex = 0;
let pages = [];
let currentContent = '';

export async function renderReader(params) {
  const app = document.getElementById('app');
  const projectId = params.id || params[0];

  app.innerHTML = `
    <!-- Top Bar -->
    <header class="topbar">
      <div class="topbar-content">
        <div class="topbar-left">
          <button id="back-btn" class="icon-btn" aria-label="Back">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M15 18l-6-6 6-6"/>
            </svg>
          </button>
          <h1 id="project-title" class="topbar-title">Loading...</h1>
        </div>
        <div class="topbar-center">
          <div id="page-indicator" class="page-indicator">Page 1 of 1</div>
        </div>
        <div class="topbar-right">
          <button id="export-btn" class="icon-btn" aria-label="Export to PDF" title="Export to PDF">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
          </button>
          <div class="topbar-divider"></div>
          <button id="prev-btn" class="icon-btn" disabled aria-label="Previous page">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
          </button>
          <button id="next-btn" class="icon-btn" disabled aria-label="Next page">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </button>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="main-content reader-content">
      <div class="reader-container">
        <div id="loading" class="loading-state">
          <div class="loading-spinner"></div>
          <span>Loading...</span>
        </div>
        <article id="markdown-content" class="prose hidden">
        </article>
      </div>
    </main>
  `;

  // Event listeners
  document.getElementById('back-btn').addEventListener('click', () => {
    cleanupSelectionPopover();
    router.navigate('/projects');
  });

  document.getElementById('prev-btn').addEventListener('click', () => navigatePage(-1));
  document.getElementById('next-btn').addEventListener('click', () => navigatePage(1));
  document.getElementById('export-btn').addEventListener('click', handleExport);

  // Initialize selection popover
  initSelectionPopover(handleAskQuestion);

  // Load project
  await loadProject(projectId);
}

async function loadProject(projectId) {
  try {
    currentProject = await getProject(projectId);
    pages = currentProject.pageOrder || [];
    currentPageIndex = 0;

    document.getElementById('project-title').textContent = currentProject.title;

    if (pages.length === 0) {
      const loading = document.getElementById('loading');
      loading.innerHTML = '<p>No pages in this project</p>';
      return;
    }

    await loadCurrentPage();
  } catch (e) {
    const loading = document.getElementById('loading');
    loading.innerHTML = `<p style="color: var(--color-error);">Failed to load project: ${e}</p>`;
    showError('Failed to load project');
  }
}

async function loadCurrentPage() {
  const loading = document.getElementById('loading');
  const content = document.getElementById('markdown-content');

  loading.classList.remove('hidden');
  content.classList.add('hidden');

  try {
    const pageName = pages[currentPageIndex];
    currentContent = await getPageContent(currentProject.id, pageName);

    content.innerHTML = renderMarkdown(currentContent);
    content.classList.remove('hidden');
    loading.classList.add('hidden');

    updateNavigation();
  } catch (e) {
    loading.innerHTML = `<p style="color: var(--color-error);">Failed to load page: ${e}</p>`;
    showError('Failed to load page');
  }
}

function updateNavigation() {
  const prevBtn = document.getElementById('prev-btn');
  const nextBtn = document.getElementById('next-btn');
  const indicator = document.getElementById('page-indicator');

  prevBtn.disabled = currentPageIndex === 0;
  nextBtn.disabled = currentPageIndex >= pages.length - 1;
  indicator.textContent = `Page ${currentPageIndex + 1} of ${pages.length}`;
}

function navigatePage(delta) {
  const newIndex = currentPageIndex + delta;
  if (newIndex >= 0 && newIndex < pages.length) {
    currentPageIndex = newIndex;
    loadCurrentPage();
    window.scrollTo(0, 0);
  }
}

async function handleExport() {
  const exportBtn = document.getElementById('export-btn');
  exportBtn.disabled = true;
  exportBtn.classList.add('loading');

  try {
    // Use Tauri's save dialog
    const { save } = window.__TAURI__.dialog;

    const filePath = await save({
      defaultPath: `${currentProject.title}.pdf`,
      filters: [{ name: 'PDF', extensions: ['pdf'] }]
    });

    if (!filePath) {
      // User cancelled
      return;
    }

    // Call backend to generate PDF at the chosen path
    await exportToPdf(currentProject.id, filePath);
    showSuccess('PDF exported successfully!');
  } catch (e) {
    showError('Failed to export: ' + e);
  } finally {
    exportBtn.disabled = false;
    exportBtn.classList.remove('loading');
  }
}

async function handleAskQuestion(selectedText, question) {
  // No blocking overlay - just inline loading in the popover

  try {
    // Send the selected text and question to backend
    const selectionInfo = {
      selectedText: selectedText,
      startLine: 0,
      endLine: 0
    };

    const result = await expandSelection(
      currentProject.id,
      pages[currentPageIndex],
      selectionInfo,
      question
    );

    // Hide popover first
    hidePopover();

    // Update content and re-render
    currentContent = result.updatedMarkdown;
    const contentEl = document.getElementById('markdown-content');
    contentEl.innerHTML = renderMarkdown(currentContent);

    // Flash the newly inserted content
    flashInsertedContent(contentEl, result.insertedContent || '');

  } catch (e) {
    hidePopoverLoading();
    showError('Failed: ' + e);
  }
}

function flashInsertedContent(contentEl, insertedContent) {
  if (!insertedContent || insertedContent.trim() === '') {
    return;
  }

  // Wait for DOM to settle
  setTimeout(() => {
    // Get all block-level elements in rendered markdown
    const elements = contentEl.querySelectorAll('p, h1, h2, h3, h4, h5, h6, li, blockquote, pre');

    // Extract text snippets from the inserted markdown content
    // Remove markdown syntax to get plain text for matching
    const plainText = insertedContent
      .replace(/^#{1,6}\s+/gm, '')  // Remove heading markers
      .replace(/\*\*([^*]+)\*\*/g, '$1')  // Remove bold
      .replace(/\*([^*]+)\*/g, '$1')  // Remove italic
      .replace(/`([^`]+)`/g, '$1')  // Remove inline code
      .replace(/^\s*[-*+]\s+/gm, '')  // Remove list markers
      .replace(/^\s*\d+\.\s+/gm, '')  // Remove numbered list markers
      .trim();

    // Split into sentences/phrases for matching
    const searchPhrases = plainText
      .split(/[.\n]+/)
      .map(s => s.trim())
      .filter(s => s.length > 10);  // Only use phrases longer than 10 chars

    let elementsToFlash = [];

    // Find elements that contain the inserted content
    elements.forEach(el => {
      const elText = el.textContent || '';

      // Check if any of the search phrases are in this element
      for (const phrase of searchPhrases) {
        if (elText.includes(phrase)) {
          elementsToFlash.push(el);
          break;
        }
      }
    });

    // Flash all identified elements
    elementsToFlash.forEach(el => flashElement(el));

    // Scroll to first flashed element
    if (elementsToFlash.length > 0) {
      setTimeout(() => {
        elementsToFlash[0].scrollIntoView({ behavior: 'smooth', block: 'center' });
      }, 100);
    }
  }, 50);
}

function flashElement(element) {
  // Add flash animation class
  element.classList.add('content-flash');

  // Remove the class after animation completes
  setTimeout(() => {
    element.classList.remove('content-flash');
  }, 2000);
}

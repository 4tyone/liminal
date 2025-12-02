import { getProject, getPageContent, expandSelection, answerQuestion, exportToPdf, listChatSessions, createChatSession, getChatSession, sendChatMessage, deleteChatSession } from '../api.js';
import { showError, showSuccess } from '../components/toast.js';
import { initSelectionPopover, cleanupSelectionPopover, hidePopover, hidePopoverLoading, showAnswer } from '../components/selection-popover.js';
import { renderMarkdown } from '../markdown.js';
import { router } from '../router.js';

const { listen } = window.__TAURI__.event;

let currentProject = null;
let currentPageIndex = 0;
let pages = [];
let currentContent = '';

// Chat state
let chatSessions = [];
let currentChatSession = null;
let chatUnlisten = null;

// Undo/Redo history
let contentHistory = [];
let historyIndex = -1;
const MAX_HISTORY = 50;

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
          <button id="undo-btn" class="icon-btn" disabled aria-label="Undo" title="Undo (Cmd+Z)">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 7v6h6"></path>
              <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
            </svg>
          </button>
          <button id="redo-btn" class="icon-btn" disabled aria-label="Redo" title="Redo (Cmd+Shift+Z)">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 7v6h-6"></path>
              <path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7"></path>
            </svg>
          </button>
          <div class="topbar-divider"></div>
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

    <!-- Table of Contents Toggle -->
    <button id="toc-toggle" class="toc-toggle" aria-label="Table of Contents">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="3" y1="6" x2="21" y2="6"></line>
        <line x1="3" y1="12" x2="15" y2="12"></line>
        <line x1="3" y1="18" x2="18" y2="18"></line>
      </svg>
    </button>

    <!-- Table of Contents Sidebar -->
    <aside id="toc-sidebar" class="toc-sidebar">
      <div class="toc-header">
        <button id="toc-close" class="icon-btn" aria-label="Close">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
      <nav id="toc-list" class="toc-list">
        <!-- Chapters listed here -->
      </nav>
    </aside>

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

    <!-- Chat Toggle Button -->
    <button id="chat-toggle" class="chat-toggle" aria-label="Chat with AI">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
      </svg>
    </button>

    <!-- Chat Panel -->
    <aside id="chat-panel" class="chat-panel">
      <div class="chat-header">
        <div class="chat-header-left">
          <select id="chat-session-select" class="chat-session-select">
            <option value="">Select chat...</option>
          </select>
          <button id="new-chat-btn" class="icon-btn" aria-label="New Chat" title="New Chat">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
        </div>
        <button id="chat-close" class="icon-btn" aria-label="Close">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>
      <div id="chat-messages" class="chat-messages">
        <div class="chat-empty">
          <p>Start a conversation to edit your learning material.</p>
          <p class="chat-hint">Try: "Add more examples to chapter 2" or "Create a summary chapter"</p>
        </div>
      </div>
      <div class="chat-input-area">
        <div id="chat-status" class="chat-status hidden"></div>
        <div class="chat-input-wrapper">
          <textarea id="chat-input" class="chat-input" placeholder="Ask to edit, add, or change content..." rows="1"></textarea>
          <button id="chat-send" class="chat-send" aria-label="Send">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="22" y1="2" x2="11" y2="13"></line>
              <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
            </svg>
          </button>
        </div>
      </div>
    </aside>
  `;

  // Event listeners
  document.getElementById('back-btn').addEventListener('click', () => {
    cleanupReader();
    router.navigate('/projects');
  });

  document.getElementById('prev-btn').addEventListener('click', () => navigatePage(-1));
  document.getElementById('next-btn').addEventListener('click', () => navigatePage(1));
  document.getElementById('export-btn').addEventListener('click', handleExport);
  document.getElementById('undo-btn').addEventListener('click', undo);
  document.getElementById('redo-btn').addEventListener('click', redo);

  // Keyboard shortcuts for undo/redo
  document.addEventListener('keydown', handleKeyboardShortcuts);

  // TOC toggle
  document.getElementById('toc-toggle').addEventListener('click', toggleToc);
  document.getElementById('toc-close').addEventListener('click', closeToc);

  // Chat panel
  document.getElementById('chat-toggle').addEventListener('click', toggleChat);
  document.getElementById('chat-close').addEventListener('click', closeChat);
  document.getElementById('new-chat-btn').addEventListener('click', handleNewChat);
  document.getElementById('chat-session-select').addEventListener('change', handleSessionChange);
  document.getElementById('chat-send').addEventListener('click', handleSendMessage);
  document.getElementById('chat-input').addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  });

  // Auto-resize chat input
  const chatInput = document.getElementById('chat-input');
  chatInput.addEventListener('input', () => {
    chatInput.style.height = 'auto';
    chatInput.style.height = Math.min(chatInput.scrollHeight, 120) + 'px';
  });

  // Listen for chat agent status events
  chatUnlisten = await listen('chat-agent-status', (event) => {
    handleChatAgentStatus(event.payload);
  });

  // Initialize selection popover with both callbacks
  initSelectionPopover(handleEditQuestion, handleAnswerQuestion);

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

    // Populate table of contents
    populateToc();

    await loadCurrentPage();
  } catch (e) {
    const loading = document.getElementById('loading');
    loading.innerHTML = `<p style="color: var(--color-error);">Failed to load project: ${e}</p>`;
    showError('Failed to load project');
  }
}

async function reloadProject() {
  try {
    // Reload project metadata to get updated page order
    currentProject = await getProject(currentProject.id);
    const oldPages = pages;
    pages = currentProject.pageOrder || [];

    // Update title in case it changed
    document.getElementById('project-title').textContent = currentProject.title;

    // Repopulate table of contents
    populateToc();

    // If a new page was added, navigate to it
    if (pages.length > oldPages.length) {
      // Find the new page (it's likely the last one)
      const newPageIndex = pages.length - 1;
      currentPageIndex = newPageIndex;
    } else if (currentPageIndex >= pages.length) {
      // If current page was deleted, go to last page
      currentPageIndex = Math.max(0, pages.length - 1);
    }

    // Reload current page content
    await loadCurrentPage();
  } catch (e) {
    console.error('Failed to reload project:', e);
    showError('Failed to reload project');
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

    // Reset history for new page
    contentHistory = [currentContent];
    historyIndex = 0;
    updateUndoRedoButtons();

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

  // Update TOC active state
  updateTocActiveState();
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

async function handleEditQuestion(selectedText, question) {
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

    // Save to history before updating
    pushToHistory(result.updatedMarkdown);

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

async function handleAnswerQuestion(selectedText, question) {
  try {
    const selectionInfo = {
      selectedText: selectedText,
      startLine: 0,
      endLine: 0
    };

    const answer = await answerQuestion(selectionInfo, question);

    // Show the answer in the popover
    showAnswer(answer);

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

// ============================================================================
// Table of Contents
// ============================================================================

function populateToc() {
  const tocList = document.getElementById('toc-list');

  // Extract chapter titles from page filenames
  const chapters = pages.map((pageName, index) => {
    // Convert filename like "01-introduction.md" to "Introduction"
    const title = pageNameToTitle(pageName);
    return { index, title, pageName };
  });

  tocList.innerHTML = chapters.map(ch => `
    <button class="toc-item ${ch.index === currentPageIndex ? 'active' : ''}" data-index="${ch.index}">
      <span class="toc-number">${ch.index + 1}</span>
      <span class="toc-title">${escapeHtml(ch.title)}</span>
    </button>
  `).join('');

  // Add click handlers
  tocList.querySelectorAll('.toc-item').forEach(item => {
    item.addEventListener('click', () => {
      const index = parseInt(item.dataset.index, 10);
      goToPage(index);
      closeToc();
    });
  });
}

function pageNameToTitle(pageName) {
  // Remove extension
  let name = pageName.replace(/\.md$/, '');
  // Remove leading number prefix like "01-"
  name = name.replace(/^\d+-/, '');
  // Convert kebab-case to Title Case
  return name
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

function updateTocActiveState() {
  const tocItems = document.querySelectorAll('.toc-item');
  tocItems.forEach(item => {
    const index = parseInt(item.dataset.index, 10);
    item.classList.toggle('active', index === currentPageIndex);
  });
}

function goToPage(index) {
  if (index >= 0 && index < pages.length) {
    currentPageIndex = index;
    loadCurrentPage();
    window.scrollTo(0, 0);
  }
}

function toggleToc() {
  const sidebar = document.getElementById('toc-sidebar');
  const toggle = document.getElementById('toc-toggle');
  sidebar.classList.toggle('open');
  toggle.classList.toggle('active');
}

function closeToc() {
  const sidebar = document.getElementById('toc-sidebar');
  const toggle = document.getElementById('toc-toggle');
  sidebar.classList.remove('open');
  toggle.classList.remove('active');
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// ============================================================================
// Undo/Redo
// ============================================================================

function pushToHistory(newContent) {
  // Remove any redo history (everything after current index)
  contentHistory = contentHistory.slice(0, historyIndex + 1);

  // Add new content
  contentHistory.push(newContent);
  historyIndex++;

  // Limit history size
  if (contentHistory.length > MAX_HISTORY) {
    contentHistory.shift();
    historyIndex--;
  }

  updateUndoRedoButtons();
}

async function undo() {
  if (historyIndex > 0) {
    historyIndex--;
    currentContent = contentHistory[historyIndex];

    // Re-render content
    const contentEl = document.getElementById('markdown-content');
    contentEl.innerHTML = renderMarkdown(currentContent);

    // Save to backend
    await saveCurrentContent();

    updateUndoRedoButtons();
    showSuccess('Undone');
  }
}

async function redo() {
  if (historyIndex < contentHistory.length - 1) {
    historyIndex++;
    currentContent = contentHistory[historyIndex];

    // Re-render content
    const contentEl = document.getElementById('markdown-content');
    contentEl.innerHTML = renderMarkdown(currentContent);

    // Save to backend
    await saveCurrentContent();

    updateUndoRedoButtons();
    showSuccess('Redone');
  }
}

function updateUndoRedoButtons() {
  const undoBtn = document.getElementById('undo-btn');
  const redoBtn = document.getElementById('redo-btn');

  if (undoBtn) {
    undoBtn.disabled = historyIndex <= 0;
  }
  if (redoBtn) {
    redoBtn.disabled = historyIndex >= contentHistory.length - 1;
  }
}

async function saveCurrentContent() {
  try {
    const { invoke } = window.__TAURI__.core;
    await invoke('save_page_content', {
      projectId: currentProject.id,
      pageName: pages[currentPageIndex],
      content: currentContent
    });
  } catch (e) {
    console.error('Failed to save content:', e);
  }
}

function handleKeyboardShortcuts(e) {
  // Check for Cmd/Ctrl + Z (undo) or Cmd/Ctrl + Shift + Z (redo)
  if ((e.metaKey || e.ctrlKey) && e.key === 'z') {
    e.preventDefault();
    if (e.shiftKey) {
      redo();
    } else {
      undo();
    }
  }
  // Also support Cmd/Ctrl + Y for redo (Windows convention)
  if ((e.metaKey || e.ctrlKey) && e.key === 'y') {
    e.preventDefault();
    redo();
  }
}

function cleanupReader() {
  cleanupSelectionPopover();
  document.removeEventListener('keydown', handleKeyboardShortcuts);
  // Reset history
  contentHistory = [];
  historyIndex = -1;
  // Clean up chat
  if (chatUnlisten) {
    chatUnlisten();
    chatUnlisten = null;
  }
  chatSessions = [];
  currentChatSession = null;
}

// ============================================================================
// Chat Panel
// ============================================================================

function toggleChat() {
  const panel = document.getElementById('chat-panel');
  const toggle = document.getElementById('chat-toggle');
  const isOpen = panel.classList.contains('open');

  if (isOpen) {
    closeChat();
  } else {
    panel.classList.add('open');
    toggle.classList.add('active');
    // Load sessions when opening for the first time
    if (chatSessions.length === 0) {
      loadChatSessions();
    }
  }
}

function closeChat() {
  const panel = document.getElementById('chat-panel');
  const toggle = document.getElementById('chat-toggle');
  panel.classList.remove('open');
  toggle.classList.remove('active');
}

async function loadChatSessions() {
  try {
    chatSessions = await listChatSessions(currentProject.id);
    updateSessionSelect();
  } catch (e) {
    console.error('Failed to load chat sessions:', e);
  }
}

function updateSessionSelect() {
  const select = document.getElementById('chat-session-select');

  select.innerHTML = chatSessions.length === 0
    ? '<option value="">No chats yet</option>'
    : chatSessions.map(s => `
        <option value="${s.id}" ${currentChatSession?.id === s.id ? 'selected' : ''}>
          ${escapeHtml(s.title)}
        </option>
      `).join('');
}

async function handleNewChat() {
  try {
    const session = await createChatSession(currentProject.id);
    chatSessions.unshift({
      id: session.id,
      title: session.title,
      messageCount: 0,
      updatedAt: session.updatedAt
    });
    currentChatSession = session;
    updateSessionSelect();
    renderChatMessages();
    document.getElementById('chat-input').focus();
  } catch (e) {
    showError('Failed to create chat: ' + e);
  }
}

async function handleSessionChange(e) {
  const sessionId = e.target.value;
  if (!sessionId) {
    currentChatSession = null;
    renderChatMessages();
    return;
  }

  try {
    currentChatSession = await getChatSession(currentProject.id, sessionId);
    renderChatMessages();
  } catch (e) {
    showError('Failed to load chat: ' + e);
  }
}

function renderChatMessages() {
  const container = document.getElementById('chat-messages');

  if (!currentChatSession || currentChatSession.messages.length === 0) {
    container.innerHTML = `
      <div class="chat-empty">
        <p>Start a conversation to edit your learning material.</p>
        <p class="chat-hint">Try: "Add more examples to chapter 2" or "Create a summary chapter"</p>
      </div>
    `;
    return;
  }

  container.innerHTML = currentChatSession.messages.map(msg => `
    <div class="chat-message chat-message-${msg.role}">
      <div class="chat-message-content">${escapeHtml(msg.content)}</div>
    </div>
  `).join('');

  // Scroll to bottom
  container.scrollTop = container.scrollHeight;
}

async function handleSendMessage() {
  const input = document.getElementById('chat-input');
  const message = input.value.trim();

  if (!message) return;

  // Create a new session if none exists
  if (!currentChatSession) {
    try {
      const session = await createChatSession(currentProject.id);
      chatSessions.unshift({
        id: session.id,
        title: session.title,
        messageCount: 0,
        updatedAt: session.updatedAt
      });
      currentChatSession = session;
      updateSessionSelect();
    } catch (e) {
      showError('Failed to create chat: ' + e);
      return;
    }
  }

  // Add user message to UI immediately
  const messagesContainer = document.getElementById('chat-messages');

  // Clear empty state if present
  const emptyState = messagesContainer.querySelector('.chat-empty');
  if (emptyState) {
    emptyState.remove();
  }

  // Add user message
  messagesContainer.innerHTML += `
    <div class="chat-message chat-message-user">
      <div class="chat-message-content">${escapeHtml(message)}</div>
    </div>
  `;

  // Clear input
  input.value = '';
  input.style.height = 'auto';

  // Disable input while processing
  input.disabled = true;
  document.getElementById('chat-send').disabled = true;

  // Scroll to bottom
  messagesContainer.scrollTop = messagesContainer.scrollHeight;

  try {
    const result = await sendChatMessage(currentProject.id, currentChatSession.id, message);

    // Add assistant response
    messagesContainer.innerHTML += `
      <div class="chat-message chat-message-assistant">
        <div class="chat-message-content">${escapeHtml(result.response)}</div>
      </div>
    `;

    // Reload session to get updated data from backend
    try {
      currentChatSession = await getChatSession(currentProject.id, currentChatSession.id);

      // Update session in local list
      const sessionIndex = chatSessions.findIndex(s => s.id === currentChatSession.id);
      if (sessionIndex !== -1) {
        chatSessions[sessionIndex].title = currentChatSession.title;
        chatSessions[sessionIndex].messageCount = currentChatSession.messages.length;
        updateSessionSelect();
      }
    } catch (e) {
      console.error('Failed to reload chat session:', e);
    }

    // If pages changed, reload the project (to update TOC) and current page
    if (result.pagesChanged) {
      await reloadProject();
      showSuccess('Content updated');
    }

    // Scroll to bottom
    messagesContainer.scrollTop = messagesContainer.scrollHeight;

  } catch (e) {
    showError('Failed: ' + e);
    // Remove the user message we optimistically added
    const lastMessage = messagesContainer.querySelector('.chat-message:last-child');
    if (lastMessage) {
      lastMessage.remove();
    }
  } finally {
    // Re-enable input
    input.disabled = false;
    document.getElementById('chat-send').disabled = false;
    input.focus();
    hideChatStatus();
  }
}

function handleChatAgentStatus(payload) {
  const statusEl = document.getElementById('chat-status');

  if (payload.status === 'thinking') {
    statusEl.textContent = 'Thinking...';
    statusEl.classList.remove('hidden');
    statusEl.classList.add('visible', 'thinking');
  } else if (payload.status === 'executing') {
    statusEl.textContent = payload.message || 'Working...';
    statusEl.classList.remove('hidden', 'thinking');
    statusEl.classList.add('visible');
  } else if (payload.status === 'complete') {
    hideChatStatus();
  }
}

function hideChatStatus() {
  const statusEl = document.getElementById('chat-status');
  statusEl.classList.remove('visible', 'thinking');
  statusEl.classList.add('hidden');
}

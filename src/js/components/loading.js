// Agent status loading component with smooth transitions

const { listen } = window.__TAURI__.event;

let statusUnlisten = null;
let currentMessage = '';

export function showGenerationLoading() {
  const agentStatus = document.getElementById('agent-status');

  // Setup agent status below input (no overlay needed)
  if (agentStatus) {
    currentMessage = '';
    agentStatus.textContent = '';
    agentStatus.classList.add('visible');
    setupAgentStatusListener(agentStatus);
  }
}

async function setupAgentStatusListener(statusEl) {
  // Clean up any existing listener
  if (statusUnlisten) {
    statusUnlisten();
    statusUnlisten = null;
  }

  try {
    statusUnlisten = await listen('agent-status', (event) => {
      const { message } = event.payload;

      if (message && message !== currentMessage) {
        animateTextTransition(statusEl, message);
        currentMessage = message;
      }
    });
  } catch (e) {
    console.error('Failed to setup agent status listener:', e);
  }
}

function animateTextTransition(statusEl, newMessage) {
  // Fade out
  statusEl.style.opacity = '0';

  // After fade out, update text and fade in
  setTimeout(() => {
    statusEl.textContent = newMessage;

    // Fade in
    requestAnimationFrame(() => {
      statusEl.style.opacity = '1';
    });
  }, 200);
}

export function showLoading(message = 'Loading...') {
  const overlay = document.getElementById('loading-overlay');
  const textEl = document.getElementById('loading-text');
  if (textEl) {
    textEl.textContent = message;
    textEl.style.opacity = '1';
  }
  if (overlay) {
    overlay.classList.remove('minimal');
    overlay.classList.add('visible');
  }
}

export function hideLoading() {
  const overlay = document.getElementById('loading-overlay');
  const agentStatus = document.getElementById('agent-status');

  if (overlay) {
    overlay.classList.remove('visible');
  }

  if (agentStatus) {
    agentStatus.classList.remove('visible');
    agentStatus.textContent = '';
  }

  // Clean up event listener
  if (statusUnlisten) {
    statusUnlisten();
    statusUnlisten = null;
  }

  currentMessage = '';
}

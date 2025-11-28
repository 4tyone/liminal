let currentSelection = null;
let onAskCallback = null;
let mousedownHandler = null;
let mouseupHandler = null;
let keyupHandler = null;
let isLoading = false;

export function initSelectionPopover(onAsk) {
  onAskCallback = onAsk;

  const popover = document.getElementById('selection-popover');
  const popoverInput = document.getElementById('popover-input');
  const popoverSubmit = document.getElementById('popover-submit');

  if (!popover || !popoverInput || !popoverSubmit) {
    return;
  }

  mouseupHandler = handleSelectionEnd;
  keyupHandler = handleSelectionEnd;
  mousedownHandler = (e) => {
    if (!popover.contains(e.target) && !isLoading) {
      hidePopover();
    }
  };

  document.addEventListener('mouseup', mouseupHandler);
  document.addEventListener('keyup', keyupHandler);
  document.addEventListener('mousedown', mousedownHandler);

  // Submit handlers
  popoverInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && popoverInput.value.trim() && !isLoading) {
      submitQuestion(popoverInput.value.trim());
    } else if (e.key === 'Escape' && !isLoading) {
      hidePopover();
    }
  });

  popoverSubmit.addEventListener('click', () => {
    if (popoverInput.value.trim() && !isLoading) {
      submitQuestion(popoverInput.value.trim());
    }
  });
}

export function cleanupSelectionPopover() {
  if (mouseupHandler) document.removeEventListener('mouseup', mouseupHandler);
  if (keyupHandler) document.removeEventListener('keyup', keyupHandler);
  if (mousedownHandler) document.removeEventListener('mousedown', mousedownHandler);
  hidePopover();
}

function handleSelectionEnd(event) {
  const selection = window.getSelection();

  if (!selection || selection.isCollapsed || selection.toString().trim() === '') {
    return;
  }

  const contentArea = document.getElementById('markdown-content');
  if (!contentArea || !contentArea.contains(selection.anchorNode)) {
    return;
  }

  const selectedText = selection.toString().trim();
  if (selectedText.length < 3) return;

  const range = selection.getRangeAt(0);
  const rect = range.getBoundingClientRect();

  currentSelection = {
    text: selectedText,
    range: range.cloneRange()
  };

  showPopover(rect, selectedText);
}

function showPopover(rect, text) {
  const popover = document.getElementById('selection-popover');
  const popoverPreview = document.getElementById('popover-preview');
  const popoverInput = document.getElementById('popover-input');

  if (!popover || !popoverPreview || !popoverInput) return;

  const truncatedText = text.length > 60 ? text.slice(0, 60) + '...' : text;
  popoverPreview.textContent = `"${truncatedText}"`;

  // Position popover centered below selection
  const popoverWidth = 320;
  const x = rect.left + rect.width / 2 - popoverWidth / 2;
  const y = rect.bottom + 12;

  // Keep within viewport
  const maxX = window.innerWidth - popoverWidth - 16;
  const adjustedX = Math.max(16, Math.min(x, maxX));

  popover.style.left = `${adjustedX}px`;
  popover.style.top = `${y}px`;
  popover.classList.add('visible');

  // Clear and focus input
  popoverInput.value = '';
  setTimeout(() => popoverInput.focus(), 50);
}

function submitQuestion(question) {
  if (currentSelection && onAskCallback) {
    showPopoverLoading();
    onAskCallback(currentSelection.text, question);
  }
}

function showPopoverLoading() {
  isLoading = true;
  const popoverSubmit = document.getElementById('popover-submit');
  const popoverInput = document.getElementById('popover-input');

  if (popoverSubmit) {
    popoverSubmit.innerHTML = `<div class="popover-spinner"></div>`;
    popoverSubmit.disabled = true;
  }
  if (popoverInput) {
    popoverInput.disabled = true;
  }
}

export function hidePopoverLoading() {
  isLoading = false;
  const popoverSubmit = document.getElementById('popover-submit');
  const popoverInput = document.getElementById('popover-input');

  if (popoverSubmit) {
    popoverSubmit.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <line x1="22" y1="2" x2="11" y2="13"></line>
      <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
    </svg>`;
    popoverSubmit.disabled = false;
  }
  if (popoverInput) {
    popoverInput.disabled = false;
    popoverInput.value = '';
  }
}

export function hidePopover() {
  const popover = document.getElementById('selection-popover');
  const popoverInput = document.getElementById('popover-input');

  hidePopoverLoading();
  if (popover) popover.classList.remove('visible');
  if (popoverInput) popoverInput.value = '';
  currentSelection = null;
  window.getSelection()?.removeAllRanges();
}

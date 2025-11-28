// Confirmation modal component

let modalElement = null;
let resolvePromise = null;

function createModal() {
  if (modalElement) return;

  modalElement = document.createElement('div');
  modalElement.className = 'confirm-modal-overlay';
  modalElement.innerHTML = `
    <div class="confirm-modal">
      <div class="confirm-modal-icon">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
      </div>
      <h3 class="confirm-modal-title"></h3>
      <p class="confirm-modal-message"></p>
      <div class="confirm-modal-actions">
        <button class="btn btn-secondary confirm-modal-cancel">Cancel</button>
        <button class="btn btn-danger confirm-modal-confirm">Delete</button>
      </div>
    </div>
  `;

  document.body.appendChild(modalElement);

  // Event listeners
  modalElement.querySelector('.confirm-modal-cancel').addEventListener('click', () => {
    hideModal();
    if (resolvePromise) resolvePromise(false);
  });

  modalElement.querySelector('.confirm-modal-confirm').addEventListener('click', () => {
    hideModal();
    if (resolvePromise) resolvePromise(true);
  });

  // Close on overlay click
  modalElement.addEventListener('click', (e) => {
    if (e.target === modalElement) {
      hideModal();
      if (resolvePromise) resolvePromise(false);
    }
  });

  // Close on Escape key
  document.addEventListener('keydown', handleKeydown);
}

function handleKeydown(e) {
  if (e.key === 'Escape' && modalElement && modalElement.classList.contains('visible')) {
    hideModal();
    if (resolvePromise) resolvePromise(false);
  }
}

function showModal() {
  if (modalElement) {
    modalElement.classList.add('visible');
  }
}

function hideModal() {
  if (modalElement) {
    modalElement.classList.remove('visible');
  }
}

/**
 * Show a confirmation dialog
 * @param {string} title - The title of the dialog
 * @param {string} message - The message to display
 * @param {string} confirmText - Text for the confirm button (default: "Delete")
 * @returns {Promise<boolean>} - Resolves to true if confirmed, false otherwise
 */
export function confirmAction(title, message, confirmText = 'Delete') {
  createModal();

  modalElement.querySelector('.confirm-modal-title').textContent = title;
  modalElement.querySelector('.confirm-modal-message').textContent = message;
  modalElement.querySelector('.confirm-modal-confirm').textContent = confirmText;

  showModal();

  return new Promise((resolve) => {
    resolvePromise = resolve;
  });
}

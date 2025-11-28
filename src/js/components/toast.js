let hideTimeout = null;

export function showToast(message, type = 'info') {
  const toast = document.getElementById('toast');
  if (!toast) return;

  // Clear any existing timeout
  if (hideTimeout) {
    clearTimeout(hideTimeout);
  }

  // Set message and type
  toast.textContent = message;
  toast.className = 'toast visible';

  if (type === 'error') {
    toast.classList.add('toast--error');
  } else if (type === 'success') {
    toast.classList.add('toast--success');
  }

  // Auto-hide after 3 seconds
  hideTimeout = setTimeout(() => {
    toast.classList.remove('visible');
  }, 3000);
}

export function showSuccess(message) {
  showToast(message, 'success');
}

export function showError(message) {
  showToast(message, 'error');
}

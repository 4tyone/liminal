// Fun loading messages for content generation
const generationMessages = [
  "Brewing knowledge...",
  "Gathering wisdom...",
  "Crafting your learning journey...",
  "Weaving concepts together...",
  "Summoning the muses...",
  "Distilling insights...",
  "Painting with ideas...",
  "Composing your chapters...",
  "Illuminating the path...",
  "Kindling curiosity...",
  "Sculpting understanding...",
  "Threading narratives...",
  "Conjuring explanations...",
  "Assembling brilliance...",
  "Nurturing concepts...",
];

let messageInterval = null;
let currentMessageIndex = 0;

export function showGenerationLoading() {
  const overlay = document.getElementById('loading-overlay');
  const textEl = document.getElementById('loading-text');

  if (!overlay || !textEl) return;

  // Start with first message
  currentMessageIndex = Math.floor(Math.random() * generationMessages.length);
  textEl.textContent = generationMessages[currentMessageIndex];
  textEl.style.transition = 'opacity 0.3s ease';

  // Cycle through messages
  messageInterval = setInterval(() => {
    textEl.style.opacity = '0';

    setTimeout(() => {
      currentMessageIndex = (currentMessageIndex + 1) % generationMessages.length;
      textEl.textContent = generationMessages[currentMessageIndex];
      textEl.style.opacity = '1';
    }, 300);
  }, 2500);

  overlay.classList.add('visible');
}

export function showLoading(message = 'Loading...') {
  const overlay = document.getElementById('loading-overlay');
  const textEl = document.getElementById('loading-text');
  if (textEl) {
    textEl.textContent = message;
    textEl.style.opacity = '1';
  }
  if (overlay) overlay.classList.add('visible');
}

export function hideLoading() {
  const overlay = document.getElementById('loading-overlay');
  if (overlay) overlay.classList.remove('visible');

  // Clear message cycling
  if (messageInterval) {
    clearInterval(messageInterval);
    messageInterval = null;
  }
}

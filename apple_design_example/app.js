/**
 * Learning Tool - Main Application
 * A minimalist, Apple-inspired learning document viewer with AI-powered explanations
 */

// ============================================
// Configuration
// ============================================
const CONFIG = {
  // Set your AI API endpoint here (e.g., OpenAI, Anthropic, or local)
  aiEndpoint: '/api/explain', // You'll need to configure this for your backend
  debounceDelay: 150,
};

// ============================================
// Sample Markdown Content (Replace with your content loading logic)
// ============================================
const sampleMarkdown = `# Introduction to Machine Learning

Machine learning is a subset of artificial intelligence that enables systems to learn and improve from experience without being explicitly programmed. It focuses on developing computer programs that can access data and use it to learn for themselves.

## How Does It Work?

The process of learning begins with observations or data, such as examples, direct experience, or instruction. The goal is to allow computers to learn automatically without human intervention and adjust actions accordingly.

Machine learning algorithms are often categorized as:

- **Supervised learning** — The algorithm learns from labeled training data and makes predictions based on that data
- **Unsupervised learning** — The algorithm finds hidden patterns in data without pre-existing labels
- **Reinforcement learning** — The algorithm learns by interacting with an environment and receiving rewards or penalties

## Key Concepts

### Training Data

Training data is a dataset used to train a machine learning model. The quality and quantity of this data directly affects the model's performance. Good training data should be representative of the real-world scenarios the model will encounter.

### Features

Features are individual measurable properties of the data being observed. Choosing informative, discriminating, and independent features is crucial for effective algorithms. Features are typically represented as a vector of values.

### Models

A model is the mathematical representation of a real-world process. In machine learning, a model is created by training an algorithm on data. Once trained, the model can make predictions or decisions based on new, unseen data.

## A Simple Example

Consider spam email detection:

\`\`\`python
# Simple spam classifier pseudocode
def classify_email(email):
    features = extract_features(email)
    probability = model.predict(features)
    
    if probability > 0.5:
        return "spam"
    else:
        return "not spam"
\`\`\`

The model learns from thousands of labeled emails to identify patterns that distinguish spam from legitimate messages.

## Why It Matters

Machine learning is transforming industries by enabling:

1. Personalized recommendations in streaming services
2. Fraud detection in financial transactions
3. Medical diagnosis assistance
4. Autonomous vehicle navigation
5. Natural language processing for chatbots

---

*The field continues to evolve rapidly, with new techniques and applications emerging constantly.*
`;

// ============================================
// State Management
// ============================================
const state = {
  markdownSource: sampleMarkdown,
  selectedText: '',
  selectedBlockId: null,
  selectionRange: null,
  isLoading: false,
};

// ============================================
// DOM Elements
// ============================================
const elements = {
  markdownContent: document.getElementById('markdownContent'),
  selectionPopover: document.getElementById('selectionPopover'),
  selectedTextPreview: document.getElementById('selectedTextPreview'),
  questionInput: document.getElementById('questionInput'),
  submitQuestion: document.getElementById('submitQuestion'),
  loadingOverlay: document.getElementById('loadingOverlay'),
  toast: document.getElementById('toast'),
  toastMessage: document.getElementById('toastMessage'),
  docTitle: document.getElementById('docTitle'),
};

// ============================================
// Markdown Rendering with Block IDs
// ============================================

/**
 * Renders markdown to HTML with block tracking
 * Each block (paragraph, heading, list item) gets a unique data-block-id
 */
function renderMarkdown(markdown) {
  // Configure marked
  marked.setOptions({
    highlight: function(code, lang) {
      if (lang && hljs.getLanguage(lang)) {
        return hljs.highlight(code, { language: lang }).value;
      }
      return code;
    },
    breaks: false,
    gfm: true,
  });

  // Custom renderer to add block IDs
  const renderer = new marked.Renderer();
  let blockId = 0;

  // Store original methods
  const originalParagraph = renderer.paragraph.bind(renderer);
  const originalHeading = renderer.heading.bind(renderer);
  const originalListitem = renderer.listitem.bind(renderer);
  const originalBlockquote = renderer.blockquote.bind(renderer);

  renderer.paragraph = (text) => {
    blockId++;
    return `<p data-block-id="${blockId}">${text}</p>\n`;
  };

  renderer.heading = (text, level) => {
    blockId++;
    return `<h${level} data-block-id="${blockId}">${text}</h${level}>\n`;
  };

  renderer.listitem = (text) => {
    blockId++;
    return `<li data-block-id="${blockId}">${text}</li>\n`;
  };

  renderer.blockquote = (quote) => {
    blockId++;
    return `<blockquote data-block-id="${blockId}">${quote}</blockquote>\n`;
  };

  marked.use({ renderer });

  return marked.parse(markdown);
}

/**
 * Updates the document display
 */
function updateDisplay() {
  elements.markdownContent.innerHTML = renderMarkdown(state.markdownSource);
}

// ============================================
// Text Selection Handling
// ============================================

/**
 * Gets the block element containing the selection
 */
function getSelectedBlock(selection) {
  if (!selection.rangeCount) return null;
  
  const range = selection.getRangeAt(0);
  let node = range.startContainer;
  
  // Walk up to find element with data-block-id
  while (node && node !== elements.markdownContent) {
    if (node.nodeType === Node.ELEMENT_NODE && node.dataset?.blockId) {
      return node;
    }
    node = node.parentNode;
  }
  
  return null;
}

/**
 * Shows the selection popover
 */
function showPopover(x, y) {
  const popover = elements.selectionPopover;
  const rect = popover.querySelector('.popover-content').getBoundingClientRect();
  
  // Position popover above selection, centered
  let posX = x - 160; // Half of popover width
  let posY = y - 10;
  
  // Keep within viewport
  const padding = 16;
  posX = Math.max(padding, Math.min(posX, window.innerWidth - 320 - padding));
  posY = Math.max(padding, posY);
  
  popover.style.left = `${posX}px`;
  popover.style.top = `${posY}px`;
  popover.classList.add('visible');
  
  // Focus input after a short delay (for animation)
  setTimeout(() => elements.questionInput.focus(), 100);
}

/**
 * Hides the selection popover
 */
function hidePopover() {
  elements.selectionPopover.classList.remove('visible');
  elements.questionInput.value = '';
  state.selectedText = '';
  state.selectedBlockId = null;
}

/**
 * Handles text selection
 */
function handleSelection() {
  const selection = window.getSelection();
  const selectedText = selection.toString().trim();
  
  if (selectedText.length < 3) {
    hidePopover();
    return;
  }
  
  const block = getSelectedBlock(selection);
  if (!block) {
    hidePopover();
    return;
  }
  
  // Store selection info
  state.selectedText = selectedText;
  state.selectedBlockId = block.dataset.blockId;
  
  // Update preview
  const previewText = selectedText.length > 100 
    ? selectedText.substring(0, 100) + '...' 
    : selectedText;
  elements.selectedTextPreview.textContent = previewText;
  
  // Get selection position
  const range = selection.getRangeAt(0);
  const rect = range.getBoundingClientRect();
  
  // Show popover above selection
  showPopover(rect.left + rect.width / 2, rect.top);
}

// Debounced selection handler
let selectionTimeout;
function debouncedSelectionHandler() {
  clearTimeout(selectionTimeout);
  selectionTimeout = setTimeout(handleSelection, CONFIG.debounceDelay);
}

// ============================================
// AI Integration
// ============================================

/**
 * Sends question to AI and injects response into markdown
 */
async function askQuestion(question) {
  if (!state.selectedText || !state.selectedBlockId) {
    showToast('Please select some text first');
    return;
  }
  
  if (!question.trim()) {
    showToast('Please enter a question');
    return;
  }
  
  setLoading(true);
  hidePopover();
  
  try {
    // Simulate AI response (replace with actual API call)
    const explanation = await getAIExplanation(state.selectedText, question);
    
    // Inject explanation into markdown
    injectExplanation(state.selectedBlockId, explanation);
    
    // Update display
    updateDisplay();
    
    // Scroll to new content
    setTimeout(() => {
      const newBlock = document.querySelector('.ai-explanation');
      if (newBlock) {
        newBlock.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    }, 100);
    
    showToast('Explanation added');
    
  } catch (error) {
    console.error('Error getting explanation:', error);
    showToast('Failed to get explanation. Please try again.');
  } finally {
    setLoading(false);
  }
}

/**
 * Gets AI explanation (mock implementation - replace with real API)
 */
async function getAIExplanation(selectedText, question) {
  // Simulate API delay
  await new Promise(resolve => setTimeout(resolve, 1500));
  
  // Mock response - replace this with actual AI API call
  // Example for OpenAI:
  /*
  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${YOUR_API_KEY}`
    },
    body: JSON.stringify({
      model: 'gpt-4',
      messages: [
        {
          role: 'system',
          content: 'You are a helpful tutor. Provide clear, concise explanations.'
        },
        {
          role: 'user',
          content: `The student selected this text: "${selectedText}"\n\nTheir question: ${question}\n\nProvide a helpful explanation.`
        }
      ]
    })
  });
  const data = await response.json();
  return data.choices[0].message.content;
  */
  
  // Mock response for demo
  const mockResponses = {
    default: `Great question about "${selectedText.substring(0, 30)}..."! 

This concept is fundamental because it establishes the foundation for understanding how systems can improve through experience. Think of it like learning to ride a bike — you don't need someone to program every muscle movement; instead, you learn through trial and error.

The key insight here is that the algorithm adjusts its internal parameters based on feedback, gradually improving its performance over time.`
  };
  
  return mockResponses.default;
}

/**
 * Injects AI explanation after the specified block in markdown source
 */
function injectExplanation(blockId, explanation) {
  const lines = state.markdownSource.split('\n');
  let currentBlockId = 0;
  let insertIndex = -1;
  
  // Simple heuristic: count non-empty lines to find block position
  // In a production app, you'd want more sophisticated source mapping
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    
    // Skip empty lines and code block contents
    if (!line || line.startsWith('```')) continue;
    
    // Count block-creating elements
    if (line.match(/^#{1,6}\s/) ||  // Headings
        line.match(/^[-*]\s/) ||     // List items
        line.match(/^>\s/) ||        // Blockquotes
        line.match(/^\d+\.\s/) ||    // Numbered lists
        (!line.startsWith('```') && line.length > 0 && !lines[i-1]?.trim())) { // Paragraphs
      currentBlockId++;
      
      if (currentBlockId == blockId) {
        // Find end of this block
        insertIndex = i + 1;
        while (insertIndex < lines.length && lines[insertIndex].trim() && 
               !lines[insertIndex].match(/^#{1,6}\s/) &&
               !lines[insertIndex].match(/^[-*]\s/) &&
               !lines[insertIndex].match(/^>\s/) &&
               !lines[insertIndex].match(/^\d+\.\s/)) {
          insertIndex++;
        }
        break;
      }
    }
  }
  
  if (insertIndex === -1) {
    insertIndex = lines.length;
  }
  
  // Format explanation as a special block
  const formattedExplanation = `\n<div class="ai-explanation">\n\n${explanation}\n\n</div>\n`;
  
  lines.splice(insertIndex, 0, formattedExplanation);
  state.markdownSource = lines.join('\n');
}

// ============================================
// UI Helpers
// ============================================

/**
 * Shows/hides loading overlay
 */
function setLoading(loading) {
  state.isLoading = loading;
  elements.loadingOverlay.classList.toggle('visible', loading);
}

/**
 * Shows a toast notification
 */
function showToast(message, duration = 3000) {
  elements.toastMessage.textContent = message;
  elements.toast.classList.add('visible');
  
  setTimeout(() => {
    elements.toast.classList.remove('visible');
  }, duration);
}

// ============================================
// Event Listeners
// ============================================

// Text selection
document.addEventListener('mouseup', (e) => {
  // Ignore if clicking inside popover
  if (elements.selectionPopover.contains(e.target)) return;
  debouncedSelectionHandler();
});

// Close popover when clicking outside
document.addEventListener('mousedown', (e) => {
  if (!elements.selectionPopover.contains(e.target) && 
      !elements.markdownContent.contains(e.target)) {
    hidePopover();
  }
});

// Submit question
elements.submitQuestion.addEventListener('click', () => {
  askQuestion(elements.questionInput.value);
});

// Enter key to submit
elements.questionInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    askQuestion(elements.questionInput.value);
  }
  if (e.key === 'Escape') {
    hidePopover();
  }
});

// Keyboard shortcut to close popover
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    hidePopover();
  }
});

// ============================================
// Initialization
// ============================================

function init() {
  console.log('Learning Tool initialized');
  updateDisplay();
  
  // Extract title from first heading
  const titleMatch = state.markdownSource.match(/^#\s+(.+)$/m);
  if (titleMatch) {
    elements.docTitle.textContent = titleMatch[1];
  }
}

// Start app
init();

// ============================================
// Export for Tauri integration
// ============================================
window.LearningTool = {
  loadMarkdown: (markdown) => {
    state.markdownSource = markdown;
    updateDisplay();
  },
  getMarkdown: () => state.markdownSource,
  setAIEndpoint: (endpoint) => {
    CONFIG.aiEndpoint = endpoint;
  },
};

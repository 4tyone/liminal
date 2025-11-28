import { marked } from 'marked';
import hljs from 'highlight.js';
import katex from 'katex';

// Configure marked with highlight.js
marked.setOptions({
  breaks: true,
  gfm: true
});

// Custom renderer for code highlighting
const renderer = {
  code(token) {
    const code = token.text || token;
    const language = token.lang || '';
    const validLang = language && hljs.getLanguage(language);
    const highlighted = validLang
      ? hljs.highlight(code, { language }).value
      : hljs.highlightAuto(code).value;
    return `<pre><code class="hljs language-${language || 'plaintext'}">${highlighted}</code></pre>`;
  }
};

marked.use({ renderer });

// Render LaTeX math using KaTeX (before markdown parsing)
function renderMath(content) {
  // Block math: $$...$$ (can span multiple lines)
  content = content.replace(/\$\$([\s\S]+?)\$\$/g, (match, math) => {
    try {
      return katex.renderToString(math.trim(), { displayMode: true, throwOnError: false });
    } catch (e) {
      return match;
    }
  });

  // Inline math: $...$  (single line only, not $$)
  content = content.replace(/(?<!\$)\$([^$\n]+)\$(?!\$)/g, (match, math) => {
    try {
      return katex.renderToString(math.trim(), { displayMode: false, throwOnError: false });
    } catch (e) {
      return match;
    }
  });

  return content;
}

export function renderMarkdown(content) {
  if (!content || typeof content !== 'string') {
    return '';
  }
  // Process math first, then markdown
  const withMath = renderMath(content);
  return marked.parse(withMath);
}

export function findTextInMarkdown(markdown, selectedText) {
  const lines = markdown.split('\n');

  // Normalize whitespace for comparison
  const normalizeText = (text) => text.replace(/\s+/g, ' ').trim();
  const normalizedSelection = normalizeText(selectedText);

  // Single line search
  for (let i = 0; i < lines.length; i++) {
    if (normalizeText(lines[i]).includes(normalizedSelection)) {
      return {
        startLine: i,
        endLine: i,
        found: true
      };
    }
  }

  // Multi-line search - find start and end lines
  let startLine = -1;
  let endLine = -1;

  for (let i = 0; i < lines.length; i++) {
    for (let j = i; j < lines.length; j++) {
      const lineRange = lines.slice(i, j + 1).join('\n');
      if (normalizeText(lineRange).includes(normalizedSelection)) {
        startLine = i;
        endLine = j;
        return {
          startLine,
          endLine,
          found: true
        };
      }

      // Don't search too far ahead (max 20 lines)
      if (j - i > 20) break;
    }
  }

  return { found: false };
}

export function insertAfterLine(markdown, lineNumber, newContent) {
  const lines = markdown.split('\n');
  lines.splice(lineNumber + 1, 0, '', newContent, '');
  return lines.join('\n');
}

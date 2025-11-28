# Learning Tool

A minimalist, Apple-inspired learning document viewer with AI-powered explanations.

## Features

- 📖 Clean markdown rendering with syntax highlighting
- ✨ Select any text and ask AI questions about it
- 💉 AI explanations are injected directly into the document
- 🎨 Sleek, iOS-inspired design

## Quick Start

### 1. Install dependencies

```bash
npm install
```

### 2. Run development server

```bash
npm run dev
```

Open http://localhost:3000

### 3. Build for Tauri (desktop app)

First, install Tauri CLI if you haven't:

```bash
npm install -g @tauri-apps/cli
```

Initialize Tauri in the project:

```bash
npm run tauri init
```

Then build:

```bash
npm run tauri build
```

## Project Structure

```
learning-tool/
├── src/
│   ├── index.html      # Main HTML structure
│   ├── styles.css      # All styling (CSS variables, typography, etc.)
│   └── app.js          # Application logic
├── package.json
├── vite.config.js
└── README.md
```

## Connecting to an AI API

The app includes a mock AI response. To connect to a real AI:

### Option 1: OpenAI

Edit `app.js` and replace the `getAIExplanation` function:

```javascript
async function getAIExplanation(selectedText, question) {
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
          content: `The student selected: "${selectedText}"\n\nQuestion: ${question}`
        }
      ]
    })
  });
  const data = await response.json();
  return data.choices[0].message.content;
}
```

### Option 2: Anthropic Claude

```javascript
async function getAIExplanation(selectedText, question) {
  const response = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': YOUR_API_KEY,
      'anthropic-version': '2023-06-01'
    },
    body: JSON.stringify({
      model: 'claude-sonnet-4-20250514',
      max_tokens: 1024,
      messages: [{
        role: 'user',
        content: `The student selected: "${selectedText}"\n\nQuestion: ${question}\n\nProvide a helpful explanation.`
      }]
    })
  });
  const data = await response.json();
  return data.content[0].text;
}
```

### Option 3: Local Backend

For production, proxy API calls through your own backend to protect API keys:

```javascript
async function getAIExplanation(selectedText, question) {
  const response = await fetch('/api/explain', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ selectedText, question })
  });
  const data = await response.json();
  return data.explanation;
}
```

## Customization

### Colors

Edit CSS variables in `styles.css`:

```css
:root {
  --color-accent: #007AFF;        /* Primary accent */
  --color-bg: #fafafa;            /* Background */
  --color-text-primary: #1d1d1f;  /* Main text */
}
```

### Typography

```css
:root {
  --font-sans: -apple-system, BlinkMacSystemFont, 'Inter', sans-serif;
  --text-base: 15px;
}
```

## Loading Custom Markdown

Use the exposed API:

```javascript
// Load markdown content
window.LearningTool.loadMarkdown(`# My Document\n\nContent here...`);

// Get current markdown (with AI additions)
const markdown = window.LearningTool.getMarkdown();
```

## Design Principles

This app follows Apple's Human Interface Guidelines:

1. **Clarity** — Clean typography, generous whitespace
2. **Deference** — Content is the focus, UI fades into background
3. **Depth** — Subtle shadows and layers create hierarchy

Key techniques:
- Soft, diffused shadows (not harsh drop shadows)
- Translucent backgrounds with backdrop blur
- Consistent 4px/8px spacing rhythm
- Subtle, quick animations (150-300ms)
- High contrast text (dark gray, not pure black)

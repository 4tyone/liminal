# Liminal

**Personalized Learning, Powered by AI**

Liminal is a desktop application that transforms any topic into comprehensive, book-quality learning material. It combines AI-powered content generation with an elegant reading experience, enabling anyone to create personalized educational content in minutes.

---

## The Problem

Learning new topics is hard. Traditional resources are either:

- **Too generic** - Textbooks and courses cover broad audiences, not your specific needs
- **Too scattered** - Blog posts, videos, and documentation spread across dozens of tabs
- **Too passive** - You read, but can't ask questions or get clarification inline
- **Too ephemeral** - Bookmarks pile up, notes get lost, knowledge fragments

Professionals, students, and curious minds deserve better. They need a way to quickly generate structured, comprehensive learning materials tailored to exactly what they want to learn—and interact with that content intelligently.

---

## The Solution

Liminal creates personalized textbooks on any topic. Type what you want to learn, select your depth level, and Liminal's AI agent writes a complete, well-structured learning guide. Then read it in a beautiful, book-like interface where you can highlight any text and ask questions—getting answers inline or having the AI expand the content directly.

**Your knowledge, your way, stored locally on your machine.**

---

## How It Works

### 1. Generate Learning Material

Enter any topic—from "Rust ownership model" to "Renaissance art history" to "Kubernetes networking"—and choose your depth:

- **Brief**: Quick overview for context
- **Standard**: Thorough coverage with examples
- **Comprehensive**: Deep dive with technical details and edge cases

Liminal's AI agent autonomously creates multiple chapters, building a progressive curriculum from fundamentals to advanced concepts. Watch as it works, seeing real-time status updates like "Creating: Introduction to Neural Networks" and "Editing: Adding code examples."

### 2. Read Like a Book

Content appears in an elegant reader designed for focused learning:

- Serif typography optimized for long-form reading
- Table of contents for quick navigation
- Syntax-highlighted code blocks
- Mathematical equations rendered beautifully
- Warm, paper-like aesthetic that's easy on the eyes

### 3. Interact with AI Inline

Select any text and a popover appears. Two modes:

**Answer Mode**: Ask a question, get a concise answer displayed right there—without leaving your reading flow or modifying the document.

**Edit Mode**: Ask the AI to expand, clarify, or add examples. The content updates seamlessly, blending into the existing material as if it was always there. A subtle highlight shows what changed.

### 4. Undo, Redo, Iterate

Made too many changes? Full undo/redo support (Cmd+Z, Cmd+Shift+Z) lets you experiment freely. Every AI edit is tracked, so you can always go back.

### 5. Export and Share

Generate a professional PDF with:
- Title page
- Properly formatted chapters
- Syntax-highlighted code
- Mathematical notation
- Clean typography ready for printing or sharing

---

## Key Features

### AI-Powered Content Generation
- **Agentic architecture**: The AI plans, writes, and refines autonomously
- **Multi-chapter output**: Complete curricula, not just single responses
- **Depth control**: Brief overviews to comprehensive deep-dives
- **Real-time progress**: Watch the AI work with live status updates

### Interactive Reading
- **Inline Q&A**: Select text, ask questions, get answers without leaving the page
- **Smart expansion**: AI adds explanations that match the document's style
- **Undo/redo**: Full history with keyboard shortcuts
- **Visual feedback**: Flash animations highlight new content

### Beautiful Typography
- Book-quality reading experience
- Markdown with full formatting support
- Syntax highlighting for 180+ programming languages
- KaTeX for mathematical equations
- Responsive layout that adapts to window size

### Project Management
- Multiple learning projects
- Chapter-based organization
- Import existing markdown folders
- Automatic saving

### Export & Sharing
- Professional PDF generation
- Maintains all formatting and code highlighting
- Ready for print or digital distribution

### Local-First Privacy
- All content stored on your machine
- No cloud sync, no data collection
- Your API key stays local
- Works offline after generation

### Auto-Updates
- Silent background update checks
- One-click installation
- Cryptographically signed releases

---

## Use Cases

### For Developers
- Generate documentation for new frameworks
- Create onboarding guides for your tech stack
- Build reference materials for complex systems
- Learn new programming languages with structured curricula

### For Students
- Create study guides tailored to your courses
- Generate practice materials at your level
- Build comprehensive notes on any subject
- Prepare for exams with focused content

### For Professionals
- Onboard to new domains quickly
- Create training materials for teams
- Document complex processes
- Build knowledge bases for specialized topics

### For the Curious
- Explore any topic that interests you
- Go from zero to competent in hours
- Build a personal library of learning materials
- Satisfy curiosity with depth

---

## Technical Details

### Stack
- **Frontend**: TypeScript, Vite, Tailwind CSS
- **Backend**: Rust with Tauri framework
- **AI**: OpenAI-compatible API (works with various providers)
- **Storage**: Local file system (no database required)

### Requirements
- macOS (Apple Silicon and Intel)
- API key for LLM provider

### Data Storage
All data stored locally:
```
~/.local/share/Liminal/
├── projects/
│   └── {project-id}/
│       ├── meta.json
│       └── pages/
│           ├── 01-introduction.md
│           └── ...
└── config.json
```

---

## Pricing

**$13/month** - Unlimited generation, all features, refundable.

---

## Get Started

1. Download Liminal for your platform
2. Enter your API key in Settings
3. Type a topic and click Generate
4. Start learning

---

## Philosophy

Liminal is built on the belief that:

1. **Learning should be personalized** - Generic content serves no one perfectly
2. **AI should augment, not replace** - You direct the learning, AI assists
3. **Privacy matters** - Your learning journey is yours alone
4. **Quality matters** - Beautiful tools inspire better work
5. **Simplicity wins** - One app, one purpose, done well

---

## The Name

*Liminal* refers to the threshold between states—the space of transformation. In learning, it's that moment when confusion gives way to understanding, when a concept finally clicks. Liminal is designed to guide you through those threshold moments, transforming topics from unknown to understood.

---

## Links

- **Website**: [liminal.wrappt.tech](https://liminal.wrappt.tech)
- **Download**: [GitHub Releases](https://github.com/4tyone/liminal-download/releases)

---

*Liminal - Transform any topic into your personal textbook.*

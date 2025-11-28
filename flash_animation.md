 Flash Animation System: Complete End-to-End Flow

  1. CSS Animation Definition

  📍 src/input.css:552-563

```css
  @keyframes contentFlash {
    0%, 100% {
      background: transparent;
    }
    20%, 80% {
      background: rgba(90, 79, 62, 0.25);
      box-shadow: 0 0 0 4px rgba(90, 79, 62, 0.15);
    }
  }

  .content-flash {
    animation: contentFlash 1.5s ease-in-out;
    border-radius: var(--radius-sm);
  }
```

  What it does: Defines a keyframe animation that flashes a warm tan
  background color with a subtle glow. Any element with class
  .content-flash will animate for 1.5 seconds.

  ---
  2. User Selects Text & Asks Question

  📍 src/js/pages/reader.js:139-187 - handleAskQuestion() function

  Step 2a: Find line numbers (lines 143-150)
  const location = findTextInMarkdown(currentContent, selectedText);
  // Returns: { startLine: 12, endLine: 17, found: true }

  Step 2b: Prepare data to send (lines 153-158)

```js

const selectionInfo = {
    startLine: location.startLine,    // e.g., 12
    endLine: location.endLine,        // e.g., 17
    selectedText: selectedText,       // The actual text
    question: question                // User's question
  };
```
  ---
  3. Frontend Sends to Backend

  📍 src/js/pages/reader.js:162-167

```javascript
  const result = await expandSelection(
    currentProject.id,
    pages[currentPageIndex],
    selectionInfo,
    question
  );
```

  This calls ⬇️

  📍 src/js/api.js:52-54 - expandSelection() wrapper
  export async function expandSelection(projectId, pageName, selection, 
  question) {
    return await invoke('expand_selection', { projectId, pageName,
  selection, question });
  }

  Which invokes Tauri IPC to call the Rust backend ⬇️

  ---
  4. Rust Backend Receives

  📍 src-tauri/src/commands/ai.rs:13-29 - expand_selection command

```rust
  #[tauri::command]
  pub async fn expand_selection(
      project_id: String,
      page_name: String,
      selection: SelectionRange,  // ← Contains startLine, endLine, 
  selectedText
      question: String,
  ) -> Result<ExpansionResult, String> {
      // ... calls ai_service
  }
```

  What the backend receives:
  📍 src-tauri/src/models/expansion.rs:5-9 - SelectionRange struct

```rust
  pub struct SelectionRange {
      pub start_line: usize,      // e.g., 12
      pub end_line: usize,        // e.g., 17
      pub selected_text: String,  // The text user selected
  }
```

  ---
  5. Backend Returns Result

  📍 src-tauri/src/models/expansion.rs:13-18 - ExpansionResult struct

```rust
  pub struct ExpansionResult {
      pub expansion_id: String,
      pub updated_markdown: String,  // ← The new markdown content
      pub inserted_content: String,
      pub insertion_line: usize,
  }
```

  ⚠️ MISSING FIELD: The struct should have:
  pub updated_lines: Option<Vec<usize>>,  // Which DOM element indices to 
  flash

  Currently, this field doesn't exist, so result.updatedLines in JavaScript
   is always undefined.

  ---
  6. Frontend Receives Result

  📍 src/js/pages/reader.js:169-176

```javascript
  // Store old content to compare
  const oldLines = currentContent.split('\n');
  const newLines = result.updatedMarkdown.split('\n');

  // Update content and re-render
  currentContent = result.updatedMarkdown;
  const contentEl = document.getElementById('markdown-content');
  contentEl.innerHTML = renderMarkdown(currentContent);
```
  ---
  7. Flash Animation Triggered

  📍 src/js/pages/reader.js:179 - Calls flashChangedLines()

  flashChangedLines(oldLines, newLines, contentEl, result.updatedLines ||
  []);

  📍 src/js/pages/reader.js:189-249 - flashChangedLines() function

```javascript
  MODE 1: Backend provides element indices (lines 193-210)
  if (updatedLineNumbers && updatedLineNumbers.length > 0) {
    // e.g., updatedLineNumbers = [5, 6, 7]
    const allElements = contentEl.querySelectorAll('p, h1, h2, h3, h4, h5, 
  h6, li, blockquote, pre, .ai-expansion');

    updatedLineNumbers.forEach(lineNum => {
      if (lineNum < allElements.length) {
        const element = allElements[lineNum];  // Get the 5th, 6th, 7th 
  element
        element.classList.add('content-flash'); // ← Adds the CSS class!
      }
    });
  }
```

  MODE 2: Fallback - Diff comparison (lines 211-246)
```js
  else {
    // Compare old markdown lines vs new markdown lines
    const changedIndices = [];
    for (let i = 0; i < maxLen; i++) {
      if (oldLines[i] !== newLines[i]) {
        changedIndices.push(i);  // e.g., [45, 46, 47, 48, 49, 50]
      }
    }

    // ⚠️ PROBLEM: changedIndices are MARKDOWN line numbers
    // But we need DOM ELEMENT indices

    // Current naive solution: Just flash the last 5 DOM elements
    const flashCount = Math.min(5, allElements.length);
    const startIdx = Math.max(0, allElements.length - flashCount);

    for (let i = startIdx; i < allElements.length; i++) {
      allElements[i].classList.add('content-flash'); // ← Adds the CSS 
  class!
    }
  }
  ```

  ---
  The Problem: Markdown Lines ≠ DOM Elements

  Example:

  Markdown (10 lines):
  Line 0: # Heading
  Line 1:
  Line 2: First paragraph.
  Line 3:
  Line 4: Second paragraph.
  Line 5:
  Line 6: - List item 1
  Line 7: - List item 2
  Line 8:
  Line 9: New content added here!

  Rendered DOM (only 5 elements):
  ```html
  <h1>Heading</h1>              <!-- Element 0 from Line 0 -->
  <p>First paragraph.</p>        <!-- Element 1 from Line 2 -->
  <p>Second paragraph.</p>       <!-- Element 2 from Line 4 -->
  <ul>
    <li>List item 1</li>         <!-- Element 3 from Line 6 -->
    <li>List item 2</li>         <!-- Element 4 from Line 7 -->
  </ul>
  <p>New content added here!</p> <!-- Element 5 from Line 9 -->
```

  If the diff says "Line 9 changed", you can't just flash DOM element 9 (it
   doesn't exist). You need to:
  1. Map markdown line 9 → DOM element 5
  2. Or have backend tell you "flash DOM element 5"

  ---
  Current Behavior

  Since result.updatedLines doesn't exist from backend:
  - Falls back to Mode 2
  - Compares markdown line by line
  - Finds many lines changed (because AI added content)
  - Ignores the specific changes
  - Just flashes the last 5 DOM elements (assuming new content is at the
  bottom)

  ---
  What You Need to Change

  Option A: Backend returns DOM element indices

  1. Update src-tauri/src/models/expansion.rs:13-18:

```rust
  pub struct ExpansionResult {
      pub expansion_id: String,
      pub updated_markdown: String,
      pub inserted_content: String,
      pub insertion_line: usize,
      pub updated_lines: Option<Vec<usize>>, // ← ADD THIS
  }
```

  2. In your AI service, calculate which DOM elements were affected:
  // After updating markdown, figure out which elements changed
  let updated_lines = Some(vec![5, 6, 7]); // These DOM elements were 
  affected

  3. Return it in the result

  Option B: Smart diff in frontend

  Build a markdown-to-DOM mapper that:
  1. Tracks which markdown lines produce which DOM elements
  2. Uses that mapping to convert changed markdown lines → DOM element
  indices
  3. Flash those specific elements

  ---
  Summary Table

  | Location                                | Purpose              | What
  It Does                                            |
  |-----------------------------------------|----------------------|-------
  --------------------------------------------------|
  | src/input.css:552-563                   | Animation definition |
  Defines the flash effect (warm tan highlight, 1.5s)     |
  | src/js/pages/reader.js:139-187          | Trigger point        | User
  asks question → sends to backend                   |
  | src/js/api.js:52-54                     | IPC wrapper          | Calls
  Rust backend via Tauri                            |
  | src-tauri/src/commands/ai.rs:13-29      | Rust command         |
  Receives request, calls AI service                      |
  | src-tauri/src/models/expansion.rs:13-18 | Return type          |
  Defines what backend sends back (missing updated_lines) |
  | src/js/pages/reader.js:179              | Trigger flash        | Calls
  flashChangedLines() with result                   |
  | src/js/pages/reader.js:189-249          | Flash logic          | Adds
  .content-flash class to DOM elements               |

  The CSS animation happens automatically once the .content-flash class is
  added to an element!


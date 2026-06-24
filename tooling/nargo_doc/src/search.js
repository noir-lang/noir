// Client-side search over the items in `window.searchIndex` (loaded from `search-index.js`).
//
// Typing in the search box replaces the page content with a list of matching items; clearing the
// box (or pressing Escape) restores the original page. No navigation happens until a result is
// clicked, so the page you were looking at is simply hidden and shown again, never rebuilt.

const searchInput = document.getElementById('search-input');
const searchToggle = document.getElementById('search-toggle');
const searchResults = document.getElementById('search-results');
const pageContent = document.getElementById('page-content');
const index = window.searchIndex || [];

// Index of the result row currently highlighted with the keyboard (-1 = none).
let activeIndex = -1;

function resultRows() {
  return searchResults.querySelectorAll('.search-result-list > li');
}

// Highlight the result row at `index` (clamped to the available rows) and scroll it into view.
function setActiveResult(index) {
  const rows = resultRows();
  if (rows.length === 0) {
    return;
  }
  if (activeIndex >= 0 && rows[activeIndex]) {
    rows[activeIndex].classList.remove('active');
  }
  activeIndex = Math.max(0, Math.min(index, rows.length - 1));
  rows[activeIndex].classList.add('active');
  rows[activeIndex].scrollIntoView({ block: 'nearest' });
}

// Show search results and hide the page content.
function showResults() {
  pageContent.hidden = true;
  searchResults.hidden = false;
}

// Hide search results and show the page content again.
function restorePage() {
  searchResults.hidden = true;
  searchResults.replaceChildren();
  pageContent.hidden = false;
}

// Scores how well an item name matches the query, or returns null if it doesn't match.
// Lower scores rank higher: an exact match beats a prefix match beats a substring match.
function score(name, query) {
  const lowerName = name.toLowerCase();
  if (lowerName === query) {
    return 0;
  }
  if (lowerName.startsWith(query)) {
    return 1;
  }
  if (lowerName.includes(query)) {
    return 2;
  }
  return null;
}

function search(query) {
  const matches = [];
  for (const item of index) {
    const itemScore = score(item.name, query);
    if (itemScore !== null) {
      matches.push({ item, score: itemScore });
    }
  }

  // Sort by score, then by name length (shorter names first), then alphabetically by path.
  matches.sort((a, b) => {
    if (a.score !== b.score) {
      return a.score - b.score;
    }
    if (a.item.name.length !== b.item.name.length) {
      return a.item.name.length - b.item.name.length;
    }
    return a.item.path.localeCompare(b.item.path);
  });

  return matches.map((match) => match.item);
}

function renderResults(query, items) {
  searchResults.replaceChildren();

  const heading = document.createElement('h1');
  heading.textContent =
    items.length === 0
      ? `No results for "${query}"`
      : `Results for "${query}"`;
  searchResults.appendChild(heading);

  if (items.length === 0) {
    return;
  }

  activeIndex = -1;

  // Each result is a row in a three-column grid: the kind ("struct", "fn", ...), the qualified
  // name, and the doc summary. Putting the kind in its own column keeps every name aligned.
  const list = document.createElement('ul');
  list.className = 'search-result-list';
  for (const item of items) {
    const li = document.createElement('li');

    const kind = document.createElement('span');
    kind.className = 'search-result-kind';
    kind.textContent = item.kind;
    li.appendChild(kind);

    // The leading path segments stay neutral; only the last segment (the item's own name) is
    // colored according to its kind.
    const link = document.createElement('a');
    link.className = 'search-result-name';
    link.href = window.docRoot + item.url;
    if (item.path) {
      const path = document.createElement('span');
      path.textContent = item.path + '::';
      link.appendChild(path);
    }
    const name = document.createElement('span');
    name.className = item.kind;
    name.textContent = item.name;
    link.appendChild(name);
    li.appendChild(link);

    const desc = document.createElement('span');
    desc.className = 'search-result-desc';
    desc.textContent = item.desc || '';
    li.appendChild(desc);

    list.appendChild(li);
  }
  searchResults.appendChild(list);
}

// Shown in the results pane before anything has been typed.
function renderGuidance() {
  activeIndex = -1;
  searchResults.replaceChildren();
  const hint = document.createElement('p');
  hint.className = 'search-guidance';
  hint.textContent = 'Type to search for items by name.';
  searchResults.appendChild(hint);
}

function onInput() {
  const query = searchInput.value.trim();
  if (query === '') {
    renderGuidance();
    return;
  }
  renderResults(query, search(query.toLowerCase()));
}

// Reveal the search box, switch to the (initially empty) results pane and turn the toggle into
// an "Exit" button.
function openSearch() {
  searchInput.hidden = false;
  searchToggle.textContent = 'Exit';
  searchToggle.classList.add('exit');
  showResults();
  renderGuidance();
  searchInput.focus();
}

// Hide the search box, clear the query, restore the page and reset the toggle.
function closeSearch() {
  searchInput.hidden = true;
  searchInput.value = '';
  searchToggle.textContent = 'Search';
  searchToggle.classList.remove('exit');
  restorePage();
}

if (searchInput && searchToggle) {
  searchInput.addEventListener('input', onInput);

  searchToggle.addEventListener('click', () => {
    if (searchInput.hidden) {
      openSearch();
    } else {
      closeSearch();
    }
  });

  searchInput.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      closeSearch();
    } else if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActiveResult(activeIndex + 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActiveResult(activeIndex - 1);
    } else if (event.key === 'Enter') {
      const rows = resultRows();
      const row = rows[activeIndex >= 0 ? activeIndex : 0];
      const link = row && row.querySelector('a');
      if (link) {
        window.location.href = link.href;
      }
    }
  });

  // Pressing "s" or "/" anywhere opens the search box, like rustdoc.
  document.addEventListener('keydown', (event) => {
    if (event.key !== '/' && event.key !== 's' && event.key !== 'S') {
      return;
    }
    const active = document.activeElement;
    const tag = active ? active.tagName : '';
    if (tag === 'INPUT' || tag === 'TEXTAREA') {
      return;
    }
    event.preventDefault();
    openSearch();
  });
}

// Client-side search over the items in `window.searchIndex` (loaded from `search-index.js`).
//
// Typing in the search box replaces the page content with a list of matching items; clearing the
// box (or pressing Escape) restores the original page. No navigation happens until a result is
// clicked, so the page you were looking at is simply hidden and shown again, never rebuilt.

const searchInput = document.getElementById('search-input');
const searchResults = document.getElementById('search-results');
const pageContent = document.getElementById('page-content');
const index = window.searchIndex || [];

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

function onInput() {
  const query = searchInput.value.trim().toLowerCase();
  if (query === '') {
    restorePage();
    return;
  }
  renderResults(searchInput.value.trim(), search(query));
  showResults();
}

if (searchInput) {
  searchInput.addEventListener('input', onInput);

  searchInput.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      searchInput.value = '';
      restorePage();
      searchInput.blur();
    }
  });

  // Pressing "/" anywhere focuses the search box, like rustdoc.
  document.addEventListener('keydown', (event) => {
    if (event.key !== '/') {
      return;
    }
    const active = document.activeElement;
    const tag = active ? active.tagName : '';
    if (tag === 'INPUT' || tag === 'TEXTAREA') {
      return;
    }
    event.preventDefault();
    searchInput.focus();
  });
}

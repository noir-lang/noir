var sidebarShown = false;

const button = document.getElementById('sidebar-toggle-button');
const main = document.getElementsByTagName('main')[0];
const sidebar = document.getElementsByTagName('nav')[0];

// When clicking on the hamburger icon, toggle between showing
// the sidebar or hiding the sidebar
button.onclick = function () {
  if (sidebarShown) {
    hideSidebar();
  } else {
    showSidebar();
  }
};

// When clicking on a sidebar link that is an anchor to the main
// content, hide the sidebar and show the main content.
const allSidebarLinks = document.querySelectorAll('.sidebar a');
for (const link of allSidebarLinks) {
  if (link.href.includes('#')) {
    link.onclick = hideSidebar;
  }
}

function showSidebar() {
  main.style.display = 'none';
  sidebar.style.display = 'block';
  sidebarShown = true;
}

function hideSidebar() {
  main.style.display = 'block';
  sidebar.style.display = 'none';
  sidebarShown = false;
}

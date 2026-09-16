// The one script the pages carry. Every control here has a page that
// works without it: the account menu is a <details>, the search box is
// a link to /search, folded forms are open by default until this hides
// them, and copy buttons sit beside text that can be selected.
(function () {
  'use strict';

  // A menu closes when the page is clicked elsewhere or Escape is pressed.
  function closeMenus(except) {
    document.querySelectorAll('details.me[open], details.more[open], details.clone[open]').forEach(function (menu) {
      if (menu !== except) menu.removeAttribute('open');
    });
  }
  document.addEventListener('click', function (event) {
    closeMenus(event.target.closest ? event.target.closest('details') : null);
  });
  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') closeMenus(null);
  });

  // Folds: a control names the element it shows or hides.
  document.querySelectorAll('[data-toggle]').forEach(function (button) {
    var target = document.getElementById(button.getAttribute('data-toggle'));
    if (!target) return;
    if (button.hasAttribute('data-toggle-closed')) target.hidden = true;
    button.setAttribute('aria-controls', target.id);
    button.setAttribute('aria-expanded', String(!target.hidden));
    button.addEventListener('click', function (event) {
      event.preventDefault();
      target.hidden = !target.hidden;
      button.setAttribute('aria-expanded', String(!target.hidden));
    });
  });

  // Repository filter: a long list narrows by text and by owner.
  (function () {
    var box = document.getElementById('repofilter');
    var list = document.getElementById('repolist');
    if (!box || !list) return;
    var input = box.querySelector('.filter');
    var owners = box.querySelectorAll('.owners button');
    var none = document.getElementById('repolist-none');
    var owner = '';
    function apply() {
      var q = input.value.trim().toLowerCase();
      var shown = 0;
      list.querySelectorAll('.item.repo').forEach(function (item) {
        var ok = (!owner || item.getAttribute('data-owner') === owner) && item.textContent.toLowerCase().indexOf(q) !== -1;
        item.hidden = !ok;
        if (ok) shown += 1;
      });
      if (none) none.hidden = shown > 0;
    }
    input.addEventListener('input', apply);
    owners.forEach(function (button) {
      button.addEventListener('click', function () {
        owner = button.getAttribute('data-owner') || '';
        owners.forEach(function (b) { b.classList.toggle('on', b === button); });
        apply();
      });
    });
  })();

  // Copy: the button names the element whose text goes to the clipboard.
  document.querySelectorAll('[data-copy]').forEach(function (button) {
    button.addEventListener('click', function (event) {
      event.preventDefault();
      var source = document.getElementById(button.getAttribute('data-copy'));
      if (!source || !navigator.clipboard) return;
      var text = source.textContent.trim();
      navigator.clipboard.writeText(text).then(function () {
        var was = button.innerHTML;
        button.textContent = 'Copied';
        setTimeout(function () { button.innerHTML = was; }, 1200);
      }).catch(function () {});
    });
  });

  // Segments: buttons that show one pane of a group. Without the
  // script every pane shows; with it, the chosen one.
  document.querySelectorAll('[data-tabs]').forEach(function (group) {
    var name = group.getAttribute('data-tabs');
    var buttons = group.querySelectorAll('[data-pane]');
    function show(button) {
      buttons.forEach(function (b) { b.classList.toggle('on', b === button); });
      document.querySelectorAll('[data-pane-of="' + name + '"]').forEach(function (pane) {
        pane.hidden = pane.id !== button.getAttribute('data-pane');
      });
    }
    buttons.forEach(function (button) {
      button.addEventListener('click', function (event) { event.preventDefault(); show(button); });
    });
    var chosen = group.querySelector('.on[data-pane]') || buttons[0];
    if (chosen) show(chosen);
  });

  // The front page: the bar takes a hairline once the page has scrolled,
  // the terminal replays its transcript as typing, and the numbers count
  // up. Each has its resting state on the page already.
  var topnav = document.getElementById('topnav');
  if (topnav) {
    window.addEventListener('scroll', function () { topnav.classList.toggle('scrolled', window.scrollY > 10); }, { passive: true });
  }
  var still = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  var term = document.getElementById('term');
  if (term && !still) {
    var lines = Array.from(term.children).map(function (span) { return [span.className, span.textContent]; });
    function render(upto, partial) {
      var out = '';
      for (var i = 0; i < upto; i++) { out += '<span class="' + lines[i][0] + '"></span>'; }
      term.innerHTML = out + '<span class="cursor"></span>';
      for (var j = 0; j < upto; j++) { term.children[j].textContent = lines[j][1]; }
      if (partial !== undefined) {
        var typed = document.createElement('span');
        typed.className = lines[upto][0];
        typed.textContent = lines[upto][1].slice(0, partial);
        term.insertBefore(typed, term.lastChild);
      }
    }
    var li = 0, ch = 0;
    function tick() {
      if (li >= lines.length) { setTimeout(function () { li = 0; ch = 0; tick(); }, 4000); return; }
      var line = lines[li];
      if (line[0] === 'c') {
        ch++; render(li, ch);
        if (ch >= line[1].length) { li++; ch = 0; setTimeout(tick, 500); } else { setTimeout(tick, 38); }
      } else {
        render(li + 1); li++;
        setTimeout(tick, line[0] === 'p' ? 300 : 700);
      }
    }
    tick();
  }
  document.querySelectorAll('[data-count]').forEach(function (el) {
    var end = +el.getAttribute('data-count'), span = el.querySelector('span');
    if (!span || still || !(end > 0)) return;
    var t0 = null;
    function step(ts) {
      if (!t0) t0 = ts;
      var p = Math.min(1, (ts - t0) / 1400);
      span.textContent = Math.round(end * (1 - Math.pow(1 - p, 3)));
      if (p < 1) requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
  });

  // The palette: ⌘K or the search box opens it; typing asks /search.json;
  // Enter opens the first hit, or the search page with the same words.
  // Dropdowns: every select becomes a button and a list in the page's
  // own type; the select stays underneath for the form and for no script.
  (function () {
    var openDd = null;
    function closeDd() {
      if (!openDd) return;
      openDd.list.hidden = true;
      openDd.button.setAttribute('aria-expanded', 'false');
      openDd = null;
    }
    document.querySelectorAll('select').forEach(function (select) {
      if (select.multiple) return;
      var dd = document.createElement('div');
      dd.className = 'dd' + (select.classList.contains('sm') ? ' sm' : '');
      var button = document.createElement('button');
      button.type = 'button';
      button.className = 'dd-btn';
      button.setAttribute('aria-haspopup', 'listbox');
      button.setAttribute('aria-expanded', 'false');
      var named = select.getAttribute('aria-label');
      if (!named && select.id) {
        var label = document.querySelector('label[for="' + select.id + '"]');
        if (label) named = label.textContent;
      }
      if (named) button.setAttribute('aria-label', named.trim());
      var text = document.createElement('span');
      text.className = 'dd-text';
      button.appendChild(text);
      button.insertAdjacentHTML('beforeend', '<svg class="ic sm" aria-hidden="true"><use href="#i-chev"></use></svg>');
      var list = document.createElement('div');
      list.className = 'dd-list';
      list.setAttribute('role', 'listbox');
      list.hidden = true;
      var opts = [];
      Array.from(select.options).forEach(function (option, index) {
        var item = document.createElement('div');
        item.className = 'dd-opt';
        item.setAttribute('role', 'option');
        item.tabIndex = -1;
        item.textContent = option.textContent;
        item.addEventListener('click', function () { choose(index); closeDd(); button.focus(); });
        list.appendChild(item);
        opts.push(item);
      });
      function paint() {
        var i = select.selectedIndex;
        text.textContent = i >= 0 ? select.options[i].textContent : '';
        opts.forEach(function (o, k) { o.setAttribute('aria-selected', String(k === i)); });
      }
      function choose(index) {
        if (select.selectedIndex === index) return;
        select.selectedIndex = index;
        paint();
        select.dispatchEvent(new Event('change', { bubbles: true }));
      }
      function open() {
        closeDd();
        list.hidden = false;
        button.setAttribute('aria-expanded', 'true');
        openDd = { list: list, button: button };
        var current = opts[Math.max(select.selectedIndex, 0)];
        if (current) current.focus();
      }
      button.addEventListener('click', function () { if (list.hidden) open(); else closeDd(); });
      button.addEventListener('keydown', function (event) {
        if (['ArrowDown', 'ArrowUp', ' ', 'Enter'].indexOf(event.key) !== -1) { event.preventDefault(); open(); }
      });
      list.addEventListener('keydown', function (event) {
        var i = opts.indexOf(document.activeElement);
        var key = event.key;
        if (key === 'ArrowDown') { event.preventDefault(); opts[Math.min(i + 1, opts.length - 1)].focus(); }
        else if (key === 'ArrowUp') { event.preventDefault(); opts[Math.max(i - 1, 0)].focus(); }
        else if (key === 'Home') { event.preventDefault(); opts[0].focus(); }
        else if (key === 'End') { event.preventDefault(); opts[opts.length - 1].focus(); }
        else if (key === 'Enter' || key === ' ') { event.preventDefault(); if (i >= 0) choose(i); closeDd(); button.focus(); }
        else if (key === 'Escape') { event.preventDefault(); closeDd(); button.focus(); }
        else if (key === 'Tab') { closeDd(); }
        else if (key.length === 1) {
          var c = key.toLowerCase();
          var starts = function (o) { return o.textContent.toLowerCase().indexOf(c) === 0; };
          var next = opts.slice(i + 1).filter(starts)[0] || opts.filter(starts)[0];
          if (next) next.focus();
        }
      });
      select.addEventListener('change', paint);
      select.classList.add('dd-native');
      select.tabIndex = -1;
      select.setAttribute('aria-hidden', 'true');
      paint();
      dd.appendChild(button);
      dd.appendChild(list);
      select.parentNode.insertBefore(dd, select.nextSibling);
    });
    document.addEventListener('click', function (event) {
      if (openDd && !(event.target.closest && event.target.closest('.dd'))) closeDd();
    });
    document.addEventListener('keydown', function (event) { if (event.key === 'Escape') closeDd(); });
  })();

  var opener = document.getElementById('palette-open');
  if (!opener) return;
  var palette = null, input = null, list = null, hits = [], timer = null;

  function build() {
    palette = document.createElement('div');
    palette.className = 'palette';
    palette.hidden = true;
    palette.innerHTML =
      '<div class="palette-box" role="dialog" aria-label="Search">' +
      '<input type="search" placeholder="Search repositories, changes, tasks, people" autocomplete="off" aria-label="Search">' +
      '<div class="palette-list"></div>' +
      '<div class="palette-foot">Enter opens the first · Esc closes · <a href="/search">all results</a></div>' +
      '</div>';
    document.body.appendChild(palette);
    input = palette.querySelector('input');
    list = palette.querySelector('.palette-list');
    palette.addEventListener('click', function (event) { if (event.target === palette) close(); });
    input.addEventListener('input', function () {
      clearTimeout(timer);
      timer = setTimeout(ask, 120);
    });
    input.addEventListener('keydown', function (event) {
      if (event.key === 'Escape') { close(); }
      if (event.key === 'Enter') {
        event.preventDefault();
        var first = list.querySelector('a');
        if (first) { window.location.href = first.getAttribute('href'); }
        else { window.location.href = '/search?q=' + encodeURIComponent(input.value); }
      }
    });
  }

  function open(event) {
    if (event) event.preventDefault();
    if (!palette) build();
    palette.hidden = false;
    input.value = '';
    list.innerHTML = '';
    input.focus();
  }

  function close() {
    if (!palette || palette.hidden) return;
    palette.hidden = true;
    opener.focus();
  }

  function ask() {
    var q = input.value.trim();
    if (!q) { list.innerHTML = ''; return; }
    fetch('/search.json?q=' + encodeURIComponent(q), { credentials: 'same-origin' })
      .then(function (r) { return r.ok ? r.json() : { hits: [] }; })
      .then(function (data) {
        if (input.value.trim() !== q) return;
        hits = data.hits || [];
        list.innerHTML = '';
        hits.slice(0, 8).forEach(function (hit) {
          var a = document.createElement('a');
          a.href = hit.href;
          a.innerHTML = '<span class="k"></span><span class="t"></span><span class="d"></span>';
          a.querySelector('.k').textContent = hit.kind;
          a.querySelector('.t').textContent = hit.label;
          a.querySelector('.d').textContent = hit.detail;
          list.appendChild(a);
        });
        if (!hits.length) {
          var none = document.createElement('div');
          none.className = 'palette-none';
          none.textContent = 'Nothing matches';
          list.appendChild(none);
        }
      })
      .catch(function () {});
  }

  opener.addEventListener('click', open);
  document.addEventListener('keydown', function (event) {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') { open(event); }
    if (event.key === 'Escape') close();
  });
})();

// An empty time-zone field starts with the browser's guess; the person
// keeps or changes it before anything is saved.
(function () {
  var zone = document.getElementById('zone');
  if (!zone || zone.value) return;
  try {
    zone.value = Intl.DateTimeFormat().resolvedOptions().timeZone || '';
  } catch (e) { /* no guess, no harm */ }
})();

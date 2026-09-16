// The front page's motion: the receipt that writes itself, the terminal
// that runs, the switch between a normal git host and this one, the three
// stories, the questions, the copy button, and which way the theme button
// points. Everything here toggles classes the stylesheet already knows;
// nothing here changes what the page says.
(function () {
  'use strict';
  const $ = (sel, root) => (root || document).querySelector(sel);
  const $$ = (sel, root) => Array.from((root || document).querySelectorAll(sel));

  // The receipt on the hero: six lines arrive one at a time, the caret
  // blinks until the last, then the stamp lands; a beat later it starts
  // over, the way the design's canvas did.
  const receiptLines = $$('.rl');
  const caret = $('.rcaret');
  const seal = $('.seal');
  let line = 0;
  function drawReceipt() {
    const n = Math.min(line, receiptLines.length);
    receiptLines.forEach((el, i) => { el.hidden = i >= n; });
    if (caret) caret.style.opacity = n < receiptLines.length ? '1' : '0';
    if (seal) seal.hidden = n < receiptLines.length;
  }
  if (receiptLines.length) {
    drawReceipt();
    setInterval(() => { line = (line + 1) % 10; drawReceipt(); }, 900);
  }

  // The terminal on the self-hosting section: nine lines, a clock that
  // counts thirteen seconds a line, green once the forge is up.
  const termLines = $$('.tl');
  const tcaret = $('.tcaret');
  const clock = $('.clock');
  let tl = 0;
  function drawTerm() {
    const n = Math.min(tl, termLines.length);
    termLines.forEach((el, i) => { el.hidden = i >= n; });
    if (tcaret) tcaret.style.opacity = n < termLines.length ? '1' : '0';
    if (clock) {
      const secs = n * 13;
      clock.textContent = '0' + Math.floor(secs / 60) + ':' + String(secs % 60).padStart(2, '0');
      clock.classList.toggle('done', n >= termLines.length);
    }
  }
  if (termLines.length) {
    drawTerm();
    setInterval(() => { tl = tl >= 13 ? 1 : tl + 1; drawTerm(); }, 1100);
  }

  // A normal git host, or Ambolt: the switch flips on its own every six
  // seconds until somebody flips it by hand.
  const cmp = $('.cmp');
  let pinned = false;
  function flip() { cmp.classList.toggle('gh'); }
  if (cmp) {
    $$('[data-act="flip"]', cmp).forEach((b) => b.addEventListener('click', () => { pinned = true; flip(); }));
    setInterval(() => { if (!pinned) flip(); }, 6000);
  }

  // Three stories, one at a time; they rotate every nine seconds until
  // one is picked.
  const story = $('.story');
  let lens = 0;
  let lensPinned = false;
  function showStory(i) {
    lens = i;
    ['s0', 's1', 's2'].forEach((c, j) => story.classList.toggle(c, j === i));
    $$('.lens', story).forEach((b) => b.classList.toggle('on', Number(b.dataset.lens) === i));
  }
  if (story) {
    $$('.lens', story).forEach((b) => b.addEventListener('click', () => { lensPinned = true; showStory(Number(b.dataset.lens)); }));
    setInterval(() => { if (!lensPinned) showStory((lens + 1) % 3); }, 9000);
  }

  // The questions: one open at a time, or none.
  $$('.faq').forEach((item) => {
    const button = $('[data-act="faq"]', item);
    if (!button) return;
    button.addEventListener('click', () => {
      const open = item.classList.contains('open');
      $$('.faq').forEach((other) => {
        other.classList.remove('open');
        const b = $('[data-act="faq"]', other);
        if (b) b.setAttribute('aria-expanded', 'false');
      });
      if (!open) {
        item.classList.add('open');
        button.setAttribute('aria-expanded', 'true');
      }
    });
  });

  // The install command, onto the clipboard.
  const copy = $('[data-act="copy"]');
  if (copy) {
    const label = $('.copy-label', copy);
    copy.addEventListener('click', () => {
      try { navigator.clipboard.writeText('cargo install --git https://ambolt.sh/git/ambolt/ambolt ambolt'); } catch (e) {}
      if (label) {
        label.textContent = 'Copied to clipboard';
        setTimeout(() => { label.textContent = 'Copy the install command'; }, 1600);
      }
    });
  }

  // The theme button posts the theme the page is not in. When nothing was
  // chosen the page follows the system, so ask the system which that is.
  const themeButton = $('[data-act="theme"]');
  if (themeButton) {
    const chosen = document.documentElement.getAttribute('data-theme');
    const dark = chosen ? chosen === 'dark' : window.matchMedia('(prefers-color-scheme: dark)').matches;
    themeButton.value = dark ? 'light' : 'dark';
    const say = dark ? 'Switch to light mode' : 'Switch to dark mode';
    themeButton.setAttribute('aria-label', say);
    themeButton.setAttribute('title', say);
  }
})();

// Sent secrets: the status keys of secrets created in this browser, so the
// sender can see whether each one was opened. Never stores the secret, its
// link, its id or its #k= key — a status key can only read the status.
(function () {
  'use strict';

  const STORAGE_KEY = 'whisper_sent';
  const MAX_ENTRIES = 20;
  const section = document.getElementById('sent-secrets');

  // Storage can throw or be empty (private mode, blocked site data): the
  // page must still work, it just shows no list.
  function load() {
    try {
      const entries = JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]');
      return Array.isArray(entries) ? entries : [];
    } catch (_) {
      return [];
    }
  }

  function save(entries) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(entries.slice(0, MAX_ENTRIES)));
    } catch (_) {}
  }

  function remember(token, expiresAt, selfDestruct) {
    if (!token) return; // older server, or tracking failed: nothing to follow
    const entry = {
      token: token,
      created_at: new Date().toISOString(),
      expires_at: expiresAt,
      self_destruct: selfDestruct,
    };
    save([entry].concat(load()));
  }

  function forget(token) {
    save(load().filter((e) => e.token !== token));
  }

  // null: receipt gone (purged or unknown) · undefined: could not check
  async function fetchStatus(token) {
    try {
      const res = await fetch('/v1/status/' + encodeURIComponent(token), { cache: 'no-store' });
      if (res.status === 404) return null;
      if (!res.ok) return undefined;
      return await res.json();
    } catch (_) {
      return undefined;
    }
  }

  function formatTime(iso) {
    const date = new Date(iso);
    const time = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    if (date.toDateString() === new Date().toDateString()) return 'today ' + time;
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' }) + ' ' + time;
  }

  function describe(status) {
    if (status === undefined) return { state: 'unknown', label: 'Unknown', detail: "couldn't check" };
    if (status.state === 'opened') return { state: 'opened', label: 'Opened', detail: formatTime(status.opened_at) };
    if (status.state === 'expired') return { state: 'expired', label: 'Expired', detail: 'never opened' };
    return { state: 'waiting', label: 'Waiting', detail: 'expires ' + formatTime(status.expires_at) };
  }

  function el(tag, className, text) {
    const node = document.createElement(tag);
    node.className = className;
    if (text) node.textContent = text;
    return node;
  }

  function row(entry, status) {
    const d = describe(status);
    const li = el('li', 'sent-row sent-row-' + d.state);
    li.append(el('span', 'sent-state', d.label), el('span', 'sent-detail', d.detail));
    if (entry.self_destruct) li.append(el('span', 'sent-tag', 'self-destruct'));
    const remove = el('button', 'sent-remove', '×');
    remove.type = 'button';
    remove.setAttribute('aria-label', 'Remove from this list');
    remove.addEventListener('click', () => {
      forget(entry.token);
      li.remove();
      if (!section.querySelector('.sent-row')) section.hidden = true;
    });
    li.append(remove);
    return li;
  }

  async function refresh() {
    const entries = load();
    if (entries.length === 0) {
      section.hidden = true;
      return;
    }
    const statuses = await Promise.all(entries.map((e) => fetchStatus(e.token)));
    const gone = entries.filter((_, i) => statuses[i] === null).map((e) => e.token);
    if (gone.length) save(load().filter((e) => !gone.includes(e.token)));

    const list = el('ul', 'sent-list');
    entries.forEach((entry, i) => {
      if (statuses[i] !== null) list.append(row(entry, statuses[i]));
    });
    section.replaceChildren(
      el('h3', 'sent-title', 'Sent secrets'),
      el('p', 'sent-hint', 'Saved in this browser only — never the secret or its link.'),
      list
    );
    section.hidden = list.childElementCount === 0;
  }

  window.whisperSent = { remember: remember };

  if (section) {
    refresh();
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible') refresh();
    });
  }
})();

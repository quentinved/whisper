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

  const MAX_NAME_LENGTH = 60;

  // The name only lives here, in this browser: it is never sent to the server.
  function remember(token, expiresAt, selfDestruct, name) {
    if (!token) return; // older server, or tracking failed: nothing to follow
    const entry = {
      token: token,
      created_at: new Date().toISOString(),
      expires_at: expiresAt,
      self_destruct: selfDestruct,
    };
    const trimmed = (name || '').trim().slice(0, MAX_NAME_LENGTH);
    if (trimmed) entry.name = trimmed;
    save([entry].concat(load()));
  }

  function forget(token) {
    save(load().filter((e) => e.token !== token));
  }

  // Naming after the fact: same local-only storage as `remember`.
  function rename(token, name) {
    const trimmed = (name || '').trim().slice(0, MAX_NAME_LENGTH);
    save(
      load().map((e) => {
        if (e.token !== token) return e;
        const copy = Object.assign({}, e);
        if (trimmed) copy.name = trimmed;
        else delete copy.name;
        return copy;
      })
    );
    return trimmed;
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

  // Same wording as the iOS app (DeliveryStatus.outcome): a self-destruct
  // secret that was opened is gone for good, a reusable one stays readable
  // until it expires.
  // `parts` are joined with " · " and never broken inside, so a narrow phone
  // wraps between "today 14:02" and "readable until …", not before "PM".
  function describe(entry, status) {
    if (status === undefined) return { state: 'unknown', label: 'Unknown', parts: ["couldn't check"] };
    if (status.state === 'waiting') {
      const parts = ['expires ' + formatTime(status.expires_at)];
      if (entry.self_destruct) parts.push('opens once');
      return { state: 'waiting', label: 'Waiting', parts: parts };
    }
    if (status.state === 'expired') return { state: 'expired', label: 'Expired', parts: ['never opened'] };
    const opened = formatTime(status.opened_at);
    if (entry.self_destruct) return { state: 'opened', label: 'Opened & deleted', parts: [opened] };
    const stillReadable = new Date(status.expires_at) > new Date();
    const after = stillReadable ? 'readable until ' + formatTime(status.expires_at) : 'expired';
    return { state: 'opened', label: 'Opened', parts: [opened, after] };
  }

  function el(tag, className, text) {
    const node = document.createElement(tag);
    node.className = className;
    if (text) node.textContent = text;
    return node;
  }

  // The name slot: the name (click to rename) or "+ Add a name".
  function nameControl(entry, rerender) {
    const button = entry.name
      ? el('button', 'sent-name', entry.name)
      : el('button', 'sent-add-name', '+ Add a name');
    button.type = 'button';
    button.title = 'Only you see this name, in this browser';
    button.setAttribute('aria-label', entry.name ? 'Rename “' + entry.name + '”' : 'Add a name only you can see');
    button.addEventListener('click', () => button.replaceWith(nameEditor(entry, rerender)));
    return button;
  }

  function nameEditor(entry, rerender) {
    const input = el('input', 'sent-name-input');
    input.type = 'text';
    input.maxLength = MAX_NAME_LENGTH;
    input.value = entry.name || '';
    input.placeholder = 'Name only you can see';
    input.autocomplete = 'off';
    input.setAttribute('aria-label', 'Name for this secret, only visible in this browser');
    let done = false;
    const finish = (keep) => {
      if (done) return; // Enter removes the input, which also fires blur
      done = true;
      if (keep) {
        const name = rename(entry.token, input.value);
        if (name) entry.name = name;
        else delete entry.name;
      }
      rerender();
    };
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        finish(true);
      } else if (e.key === 'Escape') {
        finish(false);
      }
    });
    input.addEventListener('blur', () => finish(true));
    setTimeout(() => input.focus(), 0);
    return input;
  }

  function row(entry, status) {
    const d = describe(entry, status);
    const li = el('li', 'sent-row sent-row-' + d.state);
    const body = el('div', 'sent-body');
    body.append(nameControl(entry, () => li.replaceWith(row(entry, status))));
    const line = el('div', 'sent-status');
    line.append(el('strong', 'sent-part', d.label));
    d.parts.forEach((part) => line.append(' · ', el('span', 'sent-part', part)));
    body.append(line);
    li.append(el('span', 'sent-dot'), body);
    const remove = el('button', 'sent-remove', '×');
    remove.type = 'button';
    remove.setAttribute('aria-label', 'Remove ' + (entry.name ? '“' + entry.name + '” ' : '') + 'from this list');
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
      el('p', 'sent-hint', 'Saved in this browser only — never the secret or its link. Names are only visible to you.'),
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

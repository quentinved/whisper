// Zero-knowledge reveal page.
// The decryption key lives in the URL fragment (#k=...), which the browser
// never sends to the server. The secret is only fetched (and, if single-view,
// destroyed) when the user clicks Reveal.

// ===== base64url helpers =====
function b64urlDecode(str) {
  const s = str.replace(/-/g, '+').replace(/_/g, '/');
  const padded = s.length % 4 ? s + '='.repeat(4 - (s.length % 4)) : s;
  const bin = atob(padded);
  return Uint8Array.from(bin, (c) => c.charCodeAt(0));
}

async function decryptPayload(keyB64, payloadB64) {
  const rawKey = b64urlDecode(keyB64);
  const payload = b64urlDecode(payloadB64);
  if (payload.length < 13) throw new Error('payload too short');
  const key = await crypto.subtle.importKey('raw', rawKey, { name: 'AES-GCM' }, false, ['decrypt']);
  const plaintext = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: payload.slice(0, 12) },
    key,
    payload.slice(12)
  );
  return new TextDecoder().decode(plaintext);
}

// ===== display helpers (moved verbatim from the old inline script) =====
function tryParseStructured(raw) {
  const trimmed = (raw || '').trim();
  if (!trimmed.startsWith('{') || !trimmed.endsWith('}')) return null;
  try {
    const obj = JSON.parse(trimmed);
    if (obj && typeof obj === 'object' && !Array.isArray(obj) && Object.keys(obj).length > 0) {
      return obj;
    }
  } catch (e) {}
  return null;
}

async function copyToClipboard(text) {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (e) {
    const tmp = document.createElement('textarea');
    tmp.value = text;
    tmp.style.position = 'fixed';
    tmp.style.opacity = '0';
    document.body.appendChild(tmp);
    tmp.select();
    const ok = document.execCommand('copy');
    document.body.removeChild(tmp);
    return ok;
  }
}

function flashCopied(btn, originalText) {
  btn.textContent = 'Copied!';
  btn.classList.add('success');
  setTimeout(() => {
    btn.textContent = originalText;
    btn.classList.remove('success');
  }, 2000);
}

function makeIconBtn(label, svgInner) {
  const b = document.createElement('button');
  b.type = 'button';
  b.className = 'input-icon-btn';
  b.setAttribute('aria-label', label);
  const ns = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(ns, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', 'currentColor');
  svg.setAttribute('stroke-width', '2');
  svg.innerHTML = svgInner;
  b.appendChild(svg);
  return b;
}

const EYE_SVG = '<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>';
const EYE_OFF_SVG = '<path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/>';
const COPY_SVG = '<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>';

function makeRow(key, value) {
  const stringValue = String(value);

  const row = document.createElement('div');
  row.className = 'kv-row';

  const label = document.createElement('label');
  label.className = 'kv-key';
  label.textContent = key;

  const wrap = document.createElement('div');
  wrap.className = 'kv-value-wrap';

  const input = document.createElement('input');
  input.type = 'password';
  input.readOnly = true;
  input.className = 'kv-value';
  input.value = stringValue;

  const toggleBtn = makeIconBtn('Toggle visibility for ' + key, EYE_SVG);
  toggleBtn.addEventListener('click', () => {
    const hidden = input.type === 'password';
    input.type = hidden ? 'text' : 'password';
    toggleBtn.querySelector('svg').innerHTML = hidden ? EYE_OFF_SVG : EYE_SVG;
  });

  const copyBtn = makeIconBtn('Copy ' + key, COPY_SVG);
  copyBtn.addEventListener('click', async () => {
    if (window.whisperTrack) whisperTrack('secret_value_copied', { key });
    const ok = await copyToClipboard(stringValue);
    if (ok) {
      copyBtn.classList.add('success');
      setTimeout(() => copyBtn.classList.remove('success'), 1500);
    }
  });

  wrap.appendChild(input);
  wrap.appendChild(toggleBtn);
  wrap.appendChild(copyBtn);

  row.appendChild(label);
  row.appendChild(wrap);
  return row;
}

function renderStructured(obj, container) {
  for (const [key, value] of Object.entries(obj)) {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      const section = document.createElement('div');
      section.className = 'kv-section';
      const header = document.createElement('h4');
      header.className = 'kv-section-title';
      header.textContent = key;
      section.appendChild(header);
      renderStructured(value, section);
      container.appendChild(section);
    } else {
      container.appendChild(makeRow(key, value));
    }
  }
}

// ===== state machine =====
const revealState = document.getElementById('reveal-state');
const loadingState = document.getElementById('loading-state');
const revealedState = document.getElementById('revealed-state');
const errorState = document.getElementById('error-state');

function showError(message) {
  revealState.hidden = true;
  loadingState.hidden = true;
  revealedState.hidden = true;
  errorState.hidden = false;
  document.getElementById('error-message').textContent = message;
  if (window.whisperTrack) whisperTrack('secret_retrieve_failed');
}

function displaySecret(value, selfDestruct) {
  loadingState.hidden = true;
  revealedState.hidden = false;
  if (selfDestruct) document.getElementById('self-destruct-alert').hidden = false;

  const secretInput = document.getElementById('secret-input');
  const singleView = document.getElementById('single-view');
  const structuredView = document.getElementById('structured-view');
  const structuredRows = document.getElementById('structured-rows');
  secretInput.value = value;

  let payloadKind = 'text';
  const parsed = tryParseStructured(value);
  if (parsed) {
    payloadKind = 'json';
    renderStructured(parsed, structuredRows);
    singleView.hidden = true;
    structuredView.hidden = false;

    document.getElementById('raw-json-code').textContent = JSON.stringify(parsed, null, 2);

    const fieldsView = document.getElementById('fields-view');
    const rawJsonView = document.getElementById('raw-json-view');
    const jsonModeSwitch = document.getElementById('json-mode-switch');
    jsonModeSwitch.addEventListener('click', (e) => {
      const btn = e.target.closest('.mode-switch-btn');
      if (!btn) return;
      const view = btn.dataset.view;
      fieldsView.hidden = view !== 'fields';
      rawJsonView.hidden = view !== 'raw';
      jsonModeSwitch.querySelectorAll('.mode-switch-btn').forEach((b) => {
        const active = b.dataset.view === view;
        b.classList.toggle('active', active);
        b.setAttribute('aria-selected', active ? 'true' : 'false');
      });
      if (window.whisperTrack) whisperTrack('secret_view_switched', { view });
    });

    const copyAllBtn = document.getElementById('copy-all-json');
    copyAllBtn.addEventListener('click', async () => {
      if (window.whisperTrack) whisperTrack('secret_copied', { payload_kind: 'json' });
      const ok = await copyToClipboard(value);
      if (ok) flashCopied(copyAllBtn, 'Copy JSON');
    });
  } else {
    const toggleBtn = document.getElementById('toggle-secret');
    const eyeIcon = toggleBtn.querySelector('.eye-icon');
    const eyeOffIcon = toggleBtn.querySelector('.eye-off-icon');
    toggleBtn.addEventListener('click', () => {
      const isHidden = secretInput.type === 'password';
      secretInput.type = isHidden ? 'text' : 'password';
      eyeIcon.toggleAttribute('hidden', isHidden);
      eyeOffIcon.toggleAttribute('hidden', !isHidden);
    });

    const copyBtn = document.getElementById('copy-secret');
    copyBtn.addEventListener('click', async () => {
      if (window.whisperTrack) whisperTrack('secret_copied', { payload_kind: 'text' });
      const ok = await copyToClipboard(value);
      if (ok) flashCopied(copyBtn, 'Copy Secret');
    });
  }

  if (window.whisperTrack) {
    whisperTrack('secret_retrieved', { self_destruct: !!selfDestruct, payload_kind: payloadKind });
  }
}

async function reveal() {
  if (revealBtn.disabled) return;
  revealBtn.disabled = true;

  const id = new URLSearchParams(location.search).get('shared_secret_id');
  if (!id) return showError('Missing secret ID');

  revealState.hidden = true;
  loadingState.hidden = false;

  let response;
  try {
    response = await fetch(`/secret/${encodeURIComponent(id)}?source=web`);
  } catch (e) {
    return showError('Network error — please check your connection and try again.');
  }

  if (response.status === 404) {
    return showError('Secret not found — it may have expired or already been viewed.');
  }
  if (!response.ok) {
    return showError('Something went wrong on the server. Please try again later.');
  }

  let body;
  try {
    body = await response.json();
  } catch (e) {
    return showError('The server returned an unreadable response. Please try again later.');
  }
  if (typeof body.secret !== 'string') {
    return showError('The server returned an unreadable response. Please try again later.');
  }

  if (!body.client_encrypted) {
    return displaySecret(body.secret, body.self_destruct);
  }

  const keyMatch = location.hash.match(/^#k=([A-Za-z0-9_-]+)$/);
  if (!keyMatch) {
    return showError(
      'This link is missing its decryption key (the part after #). ' +
        'Ask the sender for the complete link.'
    );
  }
  if (!window.crypto || !window.crypto.subtle) {
    return showError('Your browser does not support WebCrypto, which is required to decrypt this secret.');
  }

  try {
    const plaintext = await decryptPayload(keyMatch[1], body.secret);
    displaySecret(plaintext, body.self_destruct);
  } catch (e) {
    showError('Could not decrypt — the key in the link is wrong or the data is corrupted.');
  }
}

const revealBtn = document.getElementById('reveal-btn');
if (revealBtn) revealBtn.addEventListener('click', reveal);

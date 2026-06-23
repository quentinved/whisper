// Secret visibility toggle
const secretInput = document.getElementById('secret-input');
const secretToggle = document.getElementById('secret-toggle');
let isHidden = true;

if (secretToggle && secretInput) {
  // Start with masked text
  secretInput.style.webkitTextSecurity = 'disc';
  secretInput.style.textSecurity = 'disc';

  secretToggle.addEventListener('click', () => {
    isHidden = !isHidden;
    if (isHidden) {
      secretInput.style.webkitTextSecurity = 'disc';
      secretInput.style.textSecurity = 'disc';
      secretToggle.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>';
    } else {
      secretInput.style.webkitTextSecurity = 'none';
      secretInput.style.textSecurity = 'none';
      secretToggle.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="20" height="20"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path><line x1="1" y1="1" x2="23" y2="23"></line></svg>';
    }
  });
}

// Expiration handling
const expSelect = document.getElementById('expiration-input');
const customDiv = document.getElementById('custom-expiration');
const customInput = document.getElementById('custom-expiration-date');
const expHidden = document.getElementById('expiration');

if (expSelect) {
  var MAX_MS = 7 * 24 * 60 * 60 * 1000; // 7 days

  // Set max on custom date picker
  var maxDate = new Date(Date.now() + MAX_MS);
  customInput.max = maxDate.toISOString().slice(0, 16);
  var minDate = new Date();
  customInput.min = minDate.toISOString().slice(0, 16);

  function updateExpiration() {
    const val = expSelect.value;
    const now = new Date();
    let exp;

    customDiv.classList.toggle('visible', val === 'custom');
    if (val === 'custom') {
      if (customInput.value) {
        exp = new Date(customInput.value);
        var maxAllowed = new Date(now.getTime() + MAX_MS);
        if (exp > maxAllowed) exp = maxAllowed;
      }
    } else {
      switch (val) {
        case '6-hours': exp = new Date(now.getTime() + 6 * 60 * 60 * 1000); break;
        case '1-day': exp = new Date(now.getTime() + 24 * 60 * 60 * 1000); break;
        case '2-days': exp = new Date(now.getTime() + 48 * 60 * 60 * 1000); break;
        case '7-days': exp = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000); break;
      }
    }

    if (exp) {
      expHidden.value = Math.floor(exp.getTime() / 1000);
    } else if (val === 'custom') {
      // "Custom" selected but no date picked yet: clear the stale preset so
      // the submit guard ("Please choose a valid expiration") fires instead
      // of silently submitting an expiration the user never chose.
      expHidden.value = '';
    }
  }

  expSelect.addEventListener('change', updateExpiration);
  customInput.addEventListener('change', updateExpiration);
  updateExpiration();
}

// Copy link
const copyBtn = document.getElementById('copy-link');
const linkInput = document.getElementById('shared-link-input');
if (copyBtn && linkInput) {
  const btnText = copyBtn.querySelector('span');
  function flashCopied() {
    if (btnText) btnText.textContent = 'Copied!';
    copyBtn.classList.add('copied');
    setTimeout(() => {
      if (btnText) btnText.textContent = 'Copy Link';
      copyBtn.classList.remove('copied');
    }, 2000);
  }
  copyBtn.addEventListener('click', async () => {
    if (window.whisperTrack) whisperTrack('link_copied');
    try {
      await navigator.clipboard.writeText(linkInput.value);
    } catch (e) {
      linkInput.select();
      document.execCommand('copy');
    }
    flashCopied();
  });
}

// ===== Multiple Values mode =====
const modeSwitch = document.querySelector('.mode-switch');
const freeformView = document.getElementById('freeform-view');
const multivalueView = document.getElementById('multivalue-view');
const kvRows = document.getElementById('kv-rows');
const kvAddBtn = document.getElementById('kv-add');
const kvSections = document.getElementById('kv-sections');
const kvAddSectionBtn = document.getElementById('kv-add-section');
const multivalueError = document.getElementById('multivalue-error');
const secretTextarea = document.getElementById('secret-input');

let currentMode = 'freeform';

const X_SVG = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>';

function updateRemoveButtonsIn(container) {
  const rows = container.querySelectorAll(':scope > .kv-builder-row');
  const onlyOne = rows.length === 1;
  rows.forEach((r) => {
    r.querySelector('.kv-builder-remove').disabled = onlyOne;
  });
}

function makeKvRow() {
  const row = document.createElement('div');
  row.className = 'kv-builder-row';

  const keyInput = document.createElement('input');
  keyInput.type = 'text';
  keyInput.className = 'kv-builder-input kv-builder-key';
  keyInput.placeholder = 'Key';
  keyInput.autocomplete = 'off';

  const valueInput = document.createElement('input');
  valueInput.type = 'text';
  valueInput.className = 'kv-builder-input kv-builder-value';
  valueInput.placeholder = 'Value';
  valueInput.autocomplete = 'off';

  const removeBtn = document.createElement('button');
  removeBtn.type = 'button';
  removeBtn.className = 'kv-builder-remove';
  removeBtn.setAttribute('aria-label', 'Remove this entry');
  removeBtn.innerHTML = X_SVG;
  removeBtn.addEventListener('click', () => {
    const parent = row.parentElement;
    row.remove();
    if (parent) updateRemoveButtonsIn(parent);
  });

  row.appendChild(keyInput);
  row.appendChild(valueInput);
  row.appendChild(removeBtn);
  return row;
}

function addRowTo(container, opts) {
  const row = makeKvRow();
  container.appendChild(row);
  updateRemoveButtonsIn(container);
  if (!opts || opts.focus !== false) {
    row.querySelector('.kv-builder-key').focus();
  }
}

function makeSection() {
  const block = document.createElement('div');
  block.className = 'kv-section-block';

  const header = document.createElement('div');
  header.className = 'kv-section-header';

  const nameInput = document.createElement('input');
  nameInput.type = 'text';
  nameInput.className = 'kv-builder-input kv-section-name-input';
  nameInput.placeholder = 'Section name (e.g. database)';
  nameInput.autocomplete = 'off';

  const removeSectionBtn = document.createElement('button');
  removeSectionBtn.type = 'button';
  removeSectionBtn.className = 'kv-builder-remove';
  removeSectionBtn.setAttribute('aria-label', 'Remove this section');
  removeSectionBtn.innerHTML = X_SVG;
  removeSectionBtn.addEventListener('click', () => block.remove());

  header.appendChild(nameInput);
  header.appendChild(removeSectionBtn);

  const rows = document.createElement('div');
  rows.className = 'kv-section-rows kv-builder';

  const addRowBtn = document.createElement('button');
  addRowBtn.type = 'button';
  addRowBtn.className = 'kv-builder-add';
  addRowBtn.textContent = '+ Add value';
  addRowBtn.addEventListener('click', () => addRowTo(rows));

  block.appendChild(header);
  block.appendChild(rows);
  block.appendChild(addRowBtn);

  addRowTo(rows, { focus: false }); // start with one row, but keep focus on the section name
  return block;
}

function setMode(mode) {
  currentMode = mode;
  const isFreeform = mode === 'freeform';
  freeformView.hidden = !isFreeform;
  multivalueView.hidden = isFreeform;
  secretTextarea.required = isFreeform;
  if (multivalueError) multivalueError.hidden = true;
  modeSwitch.querySelectorAll('.mode-switch-btn').forEach((btn) => {
    const active = btn.dataset.mode === mode;
    btn.classList.toggle('active', active);
    btn.setAttribute('aria-selected', active ? 'true' : 'false');
  });
}

if (modeSwitch && kvRows) {
  addRowTo(kvRows, { focus: false }); // seed without stealing focus
  modeSwitch.addEventListener('click', (e) => {
    const btn = e.target.closest('.mode-switch-btn');
    if (!btn) return;
    setMode(btn.dataset.mode);
  });
  kvAddBtn.addEventListener('click', () => addRowTo(kvRows));
  kvAddSectionBtn.addEventListener('click', () => {
    const section = makeSection();
    kvSections.appendChild(section);
    section.querySelector('.kv-section-name-input').focus();
  });
}

function collectRowsIn(container) {
  const obj = {};
  let duplicate = null;
  container.querySelectorAll(':scope > .kv-builder-row').forEach((row) => {
    const key = row.querySelector('.kv-builder-key').value.trim();
    const value = row.querySelector('.kv-builder-value').value;
    if (!key || value === '') return;
    if (key in obj) { duplicate = duplicate || key; return; }
    obj[key] = value;
  });
  return { obj, duplicate };
}

// ===== zero-knowledge encryption =====
function b64urlEncode(bytes) {
  let s = '';
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

async function encryptSecret(plaintext) {
  const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt']);
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = new Uint8Array(
    await crypto.subtle.encrypt({ name: 'AES-GCM', iv: nonce }, key, new TextEncoder().encode(plaintext))
  );
  const rawKey = new Uint8Array(await crypto.subtle.exportKey('raw', key));
  const payload = new Uint8Array(12 + ciphertext.length);
  payload.set(nonce);
  payload.set(ciphertext, 12);
  return { keyB64: b64urlEncode(rawKey), payloadB64: b64urlEncode(payload) };
}

// ===== unified submit: validate -> encrypt in browser -> POST ciphertext =====
const form = document.getElementById('form');
const submitBtn = document.getElementById('secret-submit');

function setSubmitting(submitting) {
  const spinner = document.getElementById('submit-spinner');
  const text = submitBtn.querySelector('span');
  spinner.hidden = !submitting;
  text.textContent = submitting ? 'Encrypting...' : 'Create Secure Link';
  submitBtn.disabled = submitting;
}

function showFormError(msg) {
  // multivalue mode shows errors in its banner; freeform reuses it too
  multivalueError.textContent = msg;
  multivalueError.hidden = false;
  setSubmitting(false);
}

if (form && submitBtn) {
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (multivalueError) multivalueError.hidden = true;
    setSubmitting(true);

    // 1. Collect plaintext
    let plaintext;
    if (currentMode === 'multivalue') {
      const top = collectRowsIn(kvRows);
      if (top.duplicate) return showFormError('Duplicate key: "' + top.duplicate + '". Each key must be unique.');
      const obj = { ...top.obj };
      const seenSectionNames = new Set();
      for (const block of kvSections.querySelectorAll('.kv-section-block')) {
        const name = block.querySelector('.kv-section-name-input').value.trim();
        const section = collectRowsIn(block.querySelector('.kv-section-rows'));
        if (Object.keys(section.obj).length === 0 && !name) continue;
        if (!name) return showFormError('Section name is required (or remove the empty section).');
        if (name in obj || seenSectionNames.has(name)) return showFormError('Duplicate name: "' + name + '". Section names cannot collide with top-level keys or other sections.');
        if (section.duplicate) return showFormError('Duplicate key in section "' + name + '": "' + section.duplicate + '".');
        if (Object.keys(section.obj).length === 0) continue;
        seenSectionNames.add(name);
        obj[name] = section.obj;
      }
      if (Object.keys(obj).length === 0) return showFormError('Add at least one filled key/value pair.');
      plaintext = JSON.stringify(obj);
    } else {
      plaintext = document.getElementById('secret-input').value;
      if (!plaintext) return showFormError('Enter a secret to share.');
    }

    // 2. Validate prerequisites — never fall back to sending plaintext
    if (!window.crypto || !window.crypto.subtle) {
      return showFormError('Your browser does not support WebCrypto (required to encrypt locally). Use a modern browser over HTTPS.');
    }
    const expiration = parseInt(document.getElementById('expiration').value, 10);
    if (!expiration) return showFormError('Please choose a valid expiration.');

    // 3. Encrypt locally, POST only ciphertext
    try {
      const { keyB64, payloadB64 } = await encryptSecret(plaintext);
      const response = await fetch('/v1/ephemeral?source=web', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          payload: payloadB64,
          expiration: expiration,
          self_destruct: document.getElementById('self_destruct').checked,
        }),
      });
      if (response.status !== 201) {
        let reason = 'The server rejected the request.';
        try {
          const err = await response.json();
          if (err.error && err.error.reason) reason = err.error.reason;
        } catch (_) {}
        return showFormError(reason);
      }
      const body = await response.json();
      // The key never leaves this device except inside the fragment,
      // which browsers do not send to servers.
      window.location.assign('/?shared_secret_id=' + encodeURIComponent(body.id) + '#k=' + keyB64);
    } catch (err) {
      showFormError('Network error — please try again.');
    }
  });
}

// ===== success view: complete the displayed link with the key fragment =====
if (linkInput && location.hash.startsWith('#k=')) {
  linkInput.value = linkInput.value + location.hash;
}

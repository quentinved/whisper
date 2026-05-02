# Whisper Server

## Template Structure

The template system uses Askama with a layered structure:

- `layout.html` - Base layout template that all pages extend
- `index.html` - Main page for creating shared secrets
- `get_secret.html` - Page for retrieving shared secrets

### Template Blocks

- `head` - For adding page-specific head elements
- `popup_content` - Content shown in the popup message
- `content` - Main content area
- `scripts` - JavaScript includes

## CSS Structure

Two stylesheets are loaded by `layout.html`:

- `css/modern.css` - Theme tokens (light/dark via `data-theme`), forms, buttons, alerts, mode switch, structured-secret view
- `css/pages.css` - Overrides for legal/integration pages (wider container via `:has()`, integration cards, install buttons)

## Form Submission

The form submits a POST request to `/secret` with the secret content, expiration timestamp, and optional self-destruct flag.

Example:
```html
<form
  action="/secret"
  method="post"
>
```

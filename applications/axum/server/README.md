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

The CSS has been modularized for better maintainability:

- `css/main.css` - Main entry point that imports all other CSS files
- `css/base.css` - Basic styling, reset, and layout
- `css/forms.css` - Form elements and inputs
- `css/components.css` - UI components like alerts and popups

## Form Submission

The form submits a POST request to `/secret` with the secret content, expiration timestamp, and optional self-destruct flag.

Example:
```html
<form
  action="/secret"
  method="post"
>
```

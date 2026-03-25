import { html } from 'lit';

export const templateDigest = 'mKjZkQEA/6Y=';

/**
 * Returns the template for this component.
 * Use with LitQuidElement.renderTemplate() for CSR rendering.
 */
export function getTemplate(ctx) {
  return html`<style>
    :host {
      display: block;
      font-family: system-ui, -apple-system, sans-serif;
    }
    .card {
      padding: 1rem 1.25rem;
      border: 1px solid #e0e0e0;
      border-radius: 10px;
      background: #fff;
      box-shadow: 0 1px 3px rgba(0,0,0,.06);
    }
    .name {
      margin: 0 0 0.15rem;
      font-size: 0.95rem;
      font-weight: 600;
      color: #111;
    }
    .meta {
      margin: 0;
      font-size: 0.82rem;
      color: #888;
    }
  </style>
  <div class="card">
    <p class="name">${(ctx.firstName.charAt(0).toUpperCase()+ctx.firstName.slice(1).toLowerCase())} ${ctx.lastName}</p>
    <p class="meta">${ctx.role} &middot; ${ctx.age} yrs</p>
  </div>`;
}

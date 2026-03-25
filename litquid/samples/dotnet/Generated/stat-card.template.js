import { html } from 'lit';

export const templateDigest = 'JkSablQQO4I=';

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
    .stat {
      padding: 1.25rem;
      border: 1px solid #e0e0e0;
      border-radius: 10px;
      background: #fff;
      box-shadow: 0 1px 3px rgba(0,0,0,.06);
      text-align: center;
    }
    .count {
      display: block;
      font-size: 2rem;
      font-weight: 700;
      color: #111;
      line-height: 1;
      margin-bottom: 0.3rem;
    }
    .label {
      display: block;
      font-size: 0.85rem;
      font-weight: 600;
      color: #444;
    }
    .sub {
      display: block;
      font-size: 0.75rem;
      color: #888;
      margin-top: 0.2rem;
    }
  </style>
  <div class="stat">
    <span class="count">${ctx.count}</span>
    <span class="label">${ctx.label}</span>
    <span class="sub">${ctx.sub}</span>
  </div>`;
}

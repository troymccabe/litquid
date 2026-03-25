// Import everything from @litquid/runtime — never from 'lit' directly.
import { LitQuidElement, css, type TemplateResult } from '@litquid/runtime';
import { getTemplate } from './templates/stat-card.template.js';

/**
 * StatCard - aggregate team stat tile.
 *
 * @element stat-card
 */
export class StatCard extends LitQuidElement {
    static properties = {
        count: { type: String },
        label: { type: String },
        sub:   { type: String },
    };

    static styles = css`
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
    `;

    count = '';
    label = '';
    sub   = '';

    protected override renderTemplate(): TemplateResult {
        return getTemplate(this);
    }
}

customElements.define('stat-card', StatCard);

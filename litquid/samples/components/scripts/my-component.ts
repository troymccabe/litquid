import { css, TemplateResult } from 'lit';
import { LitQuidElement } from '@litquid/runtime';
import { getTemplate } from './templates/my-component.template.js';

/**
 * MyComponent - A sample Lit component demonstrating SSR hydration.
 *
 * @element my-component
 * @attr {String} firstName - The user's first name.
 * @attr {String} lastName - The user's last name.
 * @attr {Number} age - The user's age.
 */
export class MyComponent extends LitQuidElement {
    static properties = {
        firstName: { type: String },
        lastName: { type: String },
        age: { type: Number },
    };

    static styles = css`
        :host {
            display: block;
            font-family: sans-serif;
        }

        .user-card {
            padding: 1rem;
            border: 1px solid #ccc;
            border-radius: 8px;
            background: #f9f9f9;
        }

        h2 {
            margin: 0 0 0.5rem 0;
            color: #333;
        }

        p {
            margin: 0.25rem 0;
            color: #666;
        }

        .greeting {
            display: inline-block;
            margin-top: 0.5rem;
            color: #0066cc;
        }
    `;

    firstName = '';
    lastName = '';
    age = 0;

    protected override renderTemplate(): TemplateResult {
        return getTemplate(this);
    }
}

customElements.define('my-component', MyComponent);

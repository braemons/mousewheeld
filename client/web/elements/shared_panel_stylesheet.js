// SPDX-License-Identifier: AGPL-3.0-or-later
//
// One stylesheet, adopted by every panel's shadow root.
//
// A JS module rather than a `.css` file for the two reasons statemachined
// gives: a shadow root cannot see the page's stylesheet — which is the point of
// using one, since a console's own selectors must not reach into a panel it
// embeds — and `@import` inside a shadow root is a second round trip before
// anything renders.
//
// The tokens are statemachined's, deliberately. Two daemons' panels sit on one
// console page, and a person reading them should not have to learn two visual
// languages to know which colour means "the link is down". They are custom
// properties, which is the one hook a host page gets: a shadow root blocks
// selectors, not inherited custom properties, so a console themes a panel
// without reaching into its markup.

const STYLE_TEXT = `
  :host {
    display: block;
    /* Relative units and inheritable properties: a panel dropped into a
       console takes that page's size rather than fighting it. */
    font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
    font-size: 0.9rem;
    line-height: 1.45;
    color: #16202a;

    --panel-background: #ffffff;
    --panel-border: #d3dae1;
    --panel-heading: #4a5b6a;
    --accent: #1f6feb;
    --good: #1a7f37;
    --bad: #b42318;
    --warn: #9a6700;
    --muted: #667a8a;
    --code-font: ui-monospace, "SF Mono", "Cascadia Mono", Menlo, monospace;

    /* The chart surface and its two series, from the validated categorical
       palette's light steps. The rule the panels follow: blue is what the host
       computed, orange is what the device reported. Never a hue per chart. */
    --chart-surface: #ffffff;
    --series-host: #2a78d6;
    --series-device: #eb6834;
    --chart-grid: #e6eaee;
    --chart-zero: #c4ccd4;
    --chart-stale: #98a6b3;
  }

  section {
    background: var(--panel-background);
    border: 1px solid var(--panel-border);
    border-radius: 6px;
    padding: 0.85rem 1rem 1rem;
  }

  h2 {
    margin: 0 0 0.6rem;
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--panel-heading);
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  h3 {
    margin: 1.2rem 0 0.4rem;
    font-size: 0.85rem;
    color: var(--panel-heading);
  }
  h3:first-child { margin-top: 0; }

  p { margin: 0 0 0.6rem; }
  p.muted, .muted { color: var(--muted); }
  p.note { font-size: 0.85em; color: var(--muted); margin: 0.4rem 0 0; }

  [hidden] { display: none !important; }

  .spacer { flex: 1 1 auto; }
  .row { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }

  dl.fields {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.15rem 0.9rem;
    margin: 0;
  }
  dl.fields dt { color: var(--muted); }
  dl.fields dd { margin: 0; font-variant-numeric: tabular-nums; }

  table { width: 100%; border-collapse: collapse; font-variant-numeric: tabular-nums; }
  th, td {
    text-align: left;
    padding: 0.25rem 0.5rem 0.25rem 0;
    border-bottom: 1px solid var(--panel-border);
    vertical-align: middle;
  }
  th { color: var(--muted); font-weight: 500; font-size: 0.8rem; }
  tr:last-child td { border-bottom: none; }

  code, .mono { font-family: var(--code-font); font-size: 0.85em; }

  button {
    font: inherit;
    padding: 0.25rem 0.7rem;
    border: 1px solid var(--panel-border);
    border-radius: 4px;
    background: #f6f8fa;
    cursor: pointer;
  }
  button:hover:not(:disabled) { border-color: var(--accent); }
  button:disabled { opacity: 0.55; cursor: default; }
  button.primary { background: var(--accent); border-color: var(--accent); color: #fff; }

  input, select, textarea {
    font: inherit;
    padding: 0.2rem 0.4rem;
    border: 1px solid var(--panel-border);
    border-radius: 4px;
    background: #fff;
    color: inherit;
  }
  textarea { font-family: var(--code-font); font-size: 0.85em; width: 100%; box-sizing: border-box; }
  input[type="number"] { width: 7rem; }

  .pill {
    display: inline-block;
    padding: 0.05rem 0.45rem;
    border-radius: 999px;
    font-size: 0.8em;
    border: 1px solid currentColor;
  }
  .pill.good { color: var(--good); }
  .pill.bad { color: var(--bad); }
  .pill.warn { color: var(--warn); }
  .good { color: var(--good); }
  .bad { color: var(--bad); }
  .warn { color: var(--warn); }

  .failure {
    margin: 0 0 0.6rem;
    padding: 0.35rem 0.55rem;
    border-radius: 4px;
    border: 1px solid var(--bad);
    color: var(--bad);
    background: #fff5f4;
  }
  .failure .context { color: var(--muted); }

  .scroller { overflow: auto; border: 1px solid var(--panel-border); border-radius: 4px; }

  figure { margin: 0; }
  figcaption { color: var(--muted); font-size: 0.8rem; margin-bottom: 0.15rem; }
  svg { display: block; }

  .legend { display: flex; gap: 0.9rem; flex-wrap: wrap; font-size: 0.8rem; color: var(--muted); }
  .legend .key { display: inline-flex; align-items: center; gap: 0.3rem; }
  .legend .swatch { width: 0.7rem; height: 0.15rem; border-radius: 1px; display: inline-block; }
`;

let sheet = null;

/// Adopt the one stylesheet into `root`, constructing it on first use.
export function adoptSharedStyles(root) {
  if (sheet === null && typeof CSSStyleSheet === "function") {
    try {
      sheet = new CSSStyleSheet();
      sheet.replaceSync(STYLE_TEXT);
    } catch {
      sheet = false; // constructible stylesheets unavailable; fall through
    }
  }
  if (sheet) {
    root.adoptedStyleSheets = [...(root.adoptedStyleSheets || []), sheet];
    return;
  }
  const style = document.createElement("style");
  style.textContent = STYLE_TEXT;
  root.append(style);
}

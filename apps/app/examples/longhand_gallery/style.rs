pub const SCENE_CSS: &str = r#"
.lh-main { max-width: 1280px; }
.lh-scene { scroll-margin-top: 12px; }
.lh-desc { margin: 0 0 16px; max-width: 780px; color: var(--color-ink-muted, #64748b); }
.lh-frame { height: 780px; padding: 0; }
.lh-frame-tall { height: 900px; padding: 0; }
.lh-box { height: 640px; overflow: hidden; border: 1px solid var(--color-rule, #e2e8f0); border-radius: 12px; }
.lh-card { display: grid; gap: 12px; align-content: start; min-width: 0; padding: 16px; box-sizing: border-box; border: 1px solid var(--color-rule, #e2e8f0); border-radius: 12px; background: var(--color-surface, #fff); }
.lh-grid { display: grid; gap: 16px; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); align-items: start; }
.lh-grid-main { display: grid; gap: 16px; grid-template-columns: minmax(0, 2fr) minmax(0, 1fr); align-items: start; }
.lh-grid-3 { display: grid; gap: 16px; grid-template-columns: repeat(3, minmax(0, 1fr)); align-items: start; }
.lh-stack { display: grid; gap: 16px; min-width: 0; }
.lh-tiles { display: grid; gap: 12px; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); }
.lh-h { margin: 0; font: 600 14px/1.3 system-ui, sans-serif; }
.lh-title { font: 600 15px/1.3 system-ui, sans-serif; }
.lh-muted { margin: 0; font-size: 13px; color: var(--color-ink-muted, #64748b); }
.lh-row { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
.lh-between { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; justify-content: space-between; }
.lh-list { display: grid; gap: 8px; margin: 0; padding: 0; list-style: none; }
.lh-list li { display: flex; justify-content: space-between; align-items: baseline; gap: 12px; font-size: 14px; min-width: 0; }
.lh-btn { font: 500 13px/1 system-ui, sans-serif; min-height: 32px; padding: 0 12px; border: 1px solid var(--color-rule, #cbd5e1); border-radius: 8px; background: var(--color-surface, #fff); color: inherit; cursor: pointer; }
.lh-btn:hover { background: var(--color-raised, #f1f5f9); }
.lh-btn[aria-pressed="true"], .lh-btn--primary { background: var(--color-ink, #0f172a); color: var(--color-on-ink, #fff); border-color: transparent; }
.lh-btn:focus-visible { outline: 2px solid var(--color-ink, #0f172a); outline-offset: 2px; }
.lh-btn:disabled { opacity: 0.5; cursor: default; }
.lh-input { box-sizing: border-box; width: 100%; min-height: 34px; padding: 6px 10px; border: 1px solid var(--color-rule, #cbd5e1); border-radius: 8px; background: var(--color-surface, #fff); color: inherit; font: 14px/1.4 system-ui, sans-serif; }
textarea.lh-input { min-height: 96px; resize: vertical; }
.lh-field { display: grid; gap: 4px; font-size: 13px; color: var(--color-ink-muted, #64748b); }
.lh-bar { height: 8px; border-radius: 999px; overflow: hidden; background: var(--color-rule-soft, #e2e8f0); }
.lh-bar-fill { height: 100%; border-radius: inherit; background: var(--color-ink, #0f172a); }
.lh-bar-fill[data-tone="warn"] { background: var(--color-caution, #f59e0b); }
.lh-bar-fill[data-tone="ok"] { background: var(--color-takeable, #10b981); }
.lh-chip { display: inline-flex; align-self: flex-start; justify-self: start; align-items: center; gap: 6px; min-height: 22px; padding: 0 8px; border: 1px solid var(--color-rule, #cbd5e1); border-radius: 999px; font-size: 12px; white-space: nowrap; }
.lh-chip[data-tone="warn"] { border-color: var(--color-caution, #f59e0b); color: var(--color-caution-ink, #b45309); }
.lh-chip[data-tone="ok"] { border-color: var(--color-takeable, #10b981); color: var(--color-takeable-ink, #047857); }
.lh-chip[data-tone="fault"] { border-color: var(--color-fault, #ef4444); color: var(--color-fault-ink, #b91c1c); }
.lh-banner { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; justify-content: space-between; padding: 10px 14px; border-radius: 10px; border: 1px solid var(--color-caution, #f59e0b); background: color-mix(in srgb, var(--color-caution, #f59e0b) 12%, transparent); font-size: 14px; }
.lh-node { display: grid; gap: 4px; padding: 12px; border: 1px solid var(--color-rule, #cbd5e1); border-radius: 10px; background: var(--color-surface, #fff); min-width: 0; }
.lh-node[data-state="running"] { border-color: var(--color-ink, #0f172a); }
.lh-dot { display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--color-ink-faint, #94a3b8); margin-right: 6px; }
.lh-dot[data-state="running"] { background: var(--color-takeable, #10b981); }
.lh-dot[data-state="needs"] { background: var(--color-caution, #f59e0b); }
.lh-dot[data-state="offline"] { background: var(--color-fault, #ef4444); }
.lh-arrow { display: flex; align-items: center; justify-content: center; color: var(--color-ink-faint, #94a3b8); font-size: 18px; }
.lh-msg { max-width: 82%; padding: 10px 14px; border-radius: 12px; border: 1px solid var(--color-rule, #e2e8f0); background: var(--color-raised, #f1f5f9); font-size: 14px; }
.lh-msg-me { margin-left: auto; background: var(--color-ink, #0f172a); color: var(--color-on-ink, #fff); border-color: transparent; }
.lh-thread { display: grid; gap: 10px; align-content: start; }
.lh-check { display: flex; gap: 8px; align-items: center; font-size: 14px; }
.lh-check input { margin: 0; width: 16px; height: 16px; accent-color: var(--color-ink, #0f172a); }
.lh-job { display: grid; gap: 4px; }
.lh-kv { display: flex; justify-content: space-between; gap: 12px; font-size: 14px; }
.lh-tabs { display: flex; flex-wrap: wrap; gap: 6px; }
.lh-empty { padding: 12px; font-size: 14px; color: var(--color-ink-muted, #64748b); }
.lh-graph { display: grid; gap: 8px; grid-template-columns: minmax(0, 1fr) 24px minmax(0, 1.3fr) 24px minmax(0, 1fr); align-items: start; }
@media (max-width: 720px) {
  .lh-graph { grid-template-columns: minmax(0, 1fr); }
  .lh-arrow { transform: rotate(90deg); min-height: 24px; }
  .lh-grid, .lh-grid-main, .lh-grid-3 { grid-template-columns: minmax(0, 1fr); }
  .lh-frame { height: 820px; }
  .lh-frame-tall { height: 960px; }
  .lh-msg { max-width: 100%; }
}
"#;

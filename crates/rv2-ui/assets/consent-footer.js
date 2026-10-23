(function () {
  'use strict';
  if (window.Rv2ConsentFooter) return;
  const instances = new Map();
  function mount(options) {
    const id = options.id || 'rv2-consent';
    if (instances.has(id)) instances.get(id).destroy();
    let state = { choice: null, blocked: false, storageUnavailable: false, ...options };
    let expanded = state.choice === null;
    let destroyed = false;
    const footer = options.footer || document.querySelector('footer') || document.body;
    const panel = document.createElement('section');
    panel.className = 'rv2-consent';
    panel.id = id;
    panel.setAttribute('aria-label', options.ariaLabel || 'Optional analytics');
    const inner = document.createElement('div');
    inner.className = 'rv2-consent__inner';
    const copy = document.createElement('div');
    copy.className = 'rv2-consent__copy';
    const title = document.createElement('h2');
    title.textContent = options.title || 'Analytics preferences';
    const description = document.createElement('p');
    description.textContent = options.description || 'Optional analytics help us improve this website. Tracking stays off until you allow it.';
    const status = document.createElement('p');
    status.className = 'rv2-consent__status';
    status.setAttribute('role', 'status');
    copy.append(title, description, status);
    if (options.privacyHref) {
      const privacy = document.createElement('a');
      privacy.href = options.privacyHref;
      privacy.textContent = 'Privacy policy';
      copy.append(privacy);
    }
    const actions = document.createElement('div');
    actions.className = 'rv2-consent__actions';
    function button(label, action, className) {
      const node = document.createElement('button');
      node.type = 'button';
      node.className = className || 'rv2-consent__button';
      node.textContent = label;
      node.addEventListener('click', action);
      return node;
    }
    const reopen = button('Privacy preferences', () => {
      expanded = true;
      render();
      dismiss.focus({ preventScroll: true });
    }, 'rv2-consent-reopen');
    reopen.setAttribute('aria-controls', id);
    function collapse() {
      const returnFocus = panel.contains(document.activeElement);
      expanded = false;
      render();
      if (returnFocus) reopen.focus({ preventScroll: true });
    }
    function choose(choice) {
      if (choice === 'accepted' && state.blocked) return;
      state.choice = choice;
      collapse();
      if (typeof options.onChoice === 'function') options.onChoice(choice);
    }
    const allow = button(options.allowLabel || 'Allow analytics', () => choose('accepted'));
    const decline = button(options.declineLabel || 'Keep analytics off', () => choose('declined'));
    const withdraw = button(options.withdrawLabel || 'Withdraw consent', () => choose('declined'));
    const dismiss = button('Dismiss', () => {
      if (state.choice === null) choose('declined');
      else collapse();
    }, 'rv2-consent__dismiss');
    dismiss.setAttribute('aria-label', 'Dismiss analytics preferences');
    actions.append(allow, decline, withdraw, dismiss);
    inner.append(copy, actions);
    panel.append(inner);
    // Fixed panels must escape transformed or content-visibility footer ancestors.
    document.body.append(panel);
    footer.append(reopen);
    function render() {
      panel.hidden = !expanded;
      reopen.setAttribute('aria-expanded', String(expanded));
      allow.disabled = Boolean(state.blocked);
      allow.hidden = state.choice === 'accepted';
      decline.hidden = state.choice === 'accepted';
      withdraw.hidden = state.choice !== 'accepted';
      const trackingStatus = state.blocked
        ? 'Analytics are off because your browser requests privacy.'
        : state.choice === 'accepted' ? 'Analytics are on.' : 'Analytics are off.';
      status.textContent = trackingStatus + (state.storageUnavailable
        ? ' Your preference applies to this visit because this browser cannot save it.' : '');
    }
    const handle = {
      update(next) {
        if (destroyed) return;
        const previous = state.choice;
        state = { ...state, ...next };
        if (state.choice !== previous) collapse();
        else render();
      },
      destroy() {
        if (destroyed) return;
        destroyed = true;
        const returnFocus = panel.contains(document.activeElement);
        panel.remove();
        reopen.remove();
        if (returnFocus) {
          const target = footer.querySelector('a, button');
          if (target) target.focus({ preventScroll: true });
        }
        instances.delete(id);
      }
    };
    instances.set(id, handle);
    render();
    return handle;
  }
  window.Rv2ConsentFooter = Object.freeze({ mount });
  window.dispatchEvent(new Event('rv2-consent-ready'));
}());

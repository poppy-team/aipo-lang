// aipo-dom.js — Ultralight DOM and Fine-Grained Reactive Bridge for Aipo
// Part of the official `aipo.html` package.

(function (global) {
  const elements = new Map();
  let nextId = 1;

  const aipoDom = {
    createElement(tag, attrs = {}) {
      const id = nextId++;
      const el = document.createElement(tag);
      for (const [key, value] of Object.entries(attrs)) {
        if (key === 'class' || key === 'className') {
          el.className = value;
        } else if (key === 'value') {
          el.value = value;
        } else if (key.startsWith('on_') || key.startsWith('on')) {
          // Handled via addListener
        } else {
          el.setAttribute(key, value);
        }
      }
      elements.set(id, el);
      return id;
    },

    createText(text) {
      const id = nextId++;
      const node = document.createTextNode(text);
      elements.set(id, node);
      return id;
    },

    setText(handle, text) {
      const el = elements.get(handle);
      if (el) {
        if (el.nodeType === Node.TEXT_NODE) {
          el.data = text;
        } else {
          el.textContent = text;
        }
      }
    },

    setAttribute(handle, name, value) {
      const el = elements.get(handle);
      if (!el || el.nodeType !== Node.ELEMENT_NODE) return;
      if (name === 'class' || name === 'className') {
        el.className = value;
      } else if (name === 'value') {
        el.value = value;
      } else if (value === false || value === null || value === undefined) {
        el.removeAttribute(name);
      } else if (value === true) {
        el.setAttribute(name, '');
      } else {
        el.setAttribute(name, value);
      }
    },

    appendChild(parentHandle, childHandle) {
      const parent = elements.get(parentHandle);
      const child = elements.get(childHandle);
      if (parent && child) {
        parent.appendChild(child);
      }
    },

    removeChild(parentHandle, childHandle) {
      const parent = elements.get(parentHandle);
      const child = elements.get(childHandle);
      if (parent && child) {
        parent.removeChild(child);
      }
    },

    replaceChild(parentHandle, newChildHandle, oldChildHandle) {
      const parent = elements.get(parentHandle);
      const newChild = elements.get(newChildHandle);
      const oldChild = elements.get(oldChildHandle);
      if (parent && newChild && oldChild) {
        parent.replaceChild(newChild, oldChild);
      }
    },

    addListener(handle, eventName, callback) {
      const el = elements.get(handle);
      if (!el) return;
      const cleanEvent = eventName.replace(/^on_?/, '');
      el.addEventListener(cleanEvent, (e) => {
        const payload = {
          type: e.type,
          target: { value: e.target.value ?? "" },
          key: e.key ?? "",
          prevent_default: () => e.preventDefault(),
          stop_propagation: () => e.stopPropagation()
        };
        callback(payload);
      });
    },

    injectStyle(ruleId, cssText) {
      let sheet = document.getElementById('aipo-injected-styles');
      if (!sheet) {
        sheet = document.createElement('style');
        sheet.id = 'aipo-injected-styles';
        document.head.appendChild(sheet);
      }
      if (!sheet.textContent.includes(ruleId)) {
        sheet.textContent += `\n${cssText}`;
      }
    },

    mount(selector, rootHandle) {
      const container = document.querySelector(selector);
      const root = elements.get(rootHandle);
      if (container && root) {
        container.innerHTML = '';
        container.appendChild(root);
      }
    }
  };

  global.__aipo_dom = aipoDom;
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = aipoDom;
  }
})(typeof globalThis !== 'undefined' ? globalThis : this);

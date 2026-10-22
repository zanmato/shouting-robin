// Records the boxes a rendered page draws, for layout comparison and template
// grouping. Defines window.__shoutingRobinLayout with prepare(), capture()
// and stash().
//
// The crawler runs prepare() and stash() while the page renders, in spider's
// page workers, where scrolling can take its time. stash() writes the capture
// into a script element that arrives with the page's HTML, the way the web
// vitals do, so reading it needs no open tab. It also undoes what prepare()
// changed, so the HTML the crawl audits is the page's own.
//
// The output is compact on purpose: every crawled page sends it over CDP.
// Each node is an array, see `RawNode` in src/crawl/layout.rs.
(() => {
  if (window.__shoutingRobinLayout) return;

  const STABILIZE_CSS = `
    *, *::before, *::after {
      animation: none !important;
      transition: none !important;
      caret-color: transparent !important;
      scroll-behavior: auto !important;
    }`;

  // Subtrees that never render as layout.
  const SKIP_TAGS = new Set([
    "script", "style", "noscript", "template", "link", "meta", "head", "title", "base",
  ]);
  // Elements that count as a box even when inline, and whose insides are not walked.
  const LEAF_TAGS = new Set([
    "img", "svg", "video", "canvas", "iframe", "input", "select", "textarea", "picture", "object", "embed",
  ]);
  // Must match STYLE_PROPS in src/crawl/layout.rs, which checks it.
  const STYLE_PROPS = [
    "color", "backgroundColor", "backgroundImage", "fontFamily", "fontSize", "fontWeight",
    "lineHeight", "textAlign", "textTransform", "textDecorationLine", "borderTopWidth",
    "borderTopColor", "borderRadius", "paddingTop", "paddingBottom", "paddingLeft",
    "paddingRight", "display", "flexDirection", "opacity",
  ];
  const MIN_SIDE = 4;
  const MIN_AREA = 400;
  const FLAG_FIXED = 1;
  // Must match LAYOUT_STASH_ID in src/crawl/layout.rs.
  const STASH_ID = "__sr_layout";

  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

  function fnv32(text) {
    let hash = 0x811c9dc5;
    for (let index = 0; index < text.length; index++) {
      hash ^= text.charCodeAt(index);
      hash = Math.imul(hash, 0x01000193);
    }
    // Zero means "no text" to the reader, so a real hash never is.
    return hash >>> 0 || 1;
  }

  // The types that describe the page itself: top level entities, @graph
  // members and their mainEntity. Products listed inside a category's
  // ItemList are deliberately not included, they do not make it a product page.
  function structuredDataTypes() {
    const types = new Set();
    const add = (entity) => {
      if (!entity || typeof entity !== "object") return;
      for (const type of [].concat(entity["@type"] ?? [])) {
        if (typeof type === "string") types.add(type.split(/[/#]/).pop());
      }
    };
    for (const script of document.querySelectorAll('script[type="application/ld+json"]')) {
      let data;
      try {
        data = JSON.parse(script.textContent);
      } catch {
        continue;
      }
      for (const root of [].concat(data)) {
        if (!root || typeof root !== "object") continue;
        for (const entity of [].concat(root["@graph"] ?? root)) {
          add(entity);
          for (const main of [].concat(entity?.mainEntity ?? [])) add(main);
        }
      }
    }
    for (const element of document.querySelectorAll("[itemscope]:not([itemprop])")) {
      const type = element.getAttribute("itemtype");
      if (type) types.add(type.trim().split(/[/#]/).pop());
    }
    return [...types].filter(Boolean).sort();
  }

  // Consent banners cover the page and often lock its scrolling, which
  // would make every capture show the banner instead of the page.
  function acceptCookies() {
    const pattern = /^(godkänn alla|acceptera alla|tillåt alla|godkänn|acceptera|accept all|allow all|accept|agree|jag godkänner|ok)\b/i;
    const roots = [document];
    for (let index = 0; index < roots.length; index++) {
      for (const element of roots[index].querySelectorAll("*")) {
        if (element.shadowRoot) roots.push(element.shadowRoot);
      }
    }
    for (const root of roots) {
      for (const button of root.querySelectorAll('button, a[role="button"], [role="button"]')) {
        const text = button.textContent.trim().replace(/\s+/g, " ");
        if (text.length <= 40 && pattern.test(text) && button.getClientRects().length > 0) {
          button.click();
          return true;
        }
      }
    }
    return false;
  }

  const changed = { style: null, images: [] };

  async function prepare({ scroll = false } = {}) {
    if (acceptCookies()) await sleep(500);
    const style = document.createElement("style");
    style.textContent = STABILIZE_CSS;
    document.head.append(style);
    changed.style = style;
    for (const image of document.querySelectorAll('img[loading="lazy"]')) {
      image.loading = "eager";
      changed.images.push(image);
    }
    if (scroll) {
      const limit = Math.min(document.documentElement.scrollHeight, 40000);
      for (let y = 0; y < limit; y += 700) {
        window.scrollTo(0, y);
        await sleep(150);
      }
      window.scrollTo(0, document.documentElement.scrollHeight);
      await sleep(600);
      window.scrollTo(0, 0);
      await sleep(800);
    }
    await document.fonts.ready;
    api.prepared = true;
  }

  function capture() {
    // The layout viewport. innerWidth can report the window instead.
    const viewportWidth = document.documentElement.clientWidth || window.innerWidth;
    const scrollX = window.scrollX;
    const scrollY = window.scrollY;
    const nodes = [];
    const texts = [];
    const styles = [];
    const styleIndex = new Map();
    // Element to node index, for attaching text to the nearest box.
    const owner = new Map();
    const hidden = new Set();

    function styleOf(computed) {
      const values = STYLE_PROPS.map((prop) => {
        const value = computed[prop];
        return prop === "backgroundImage" && value.length > 120 ? `url#${fnv32(value)}` : value;
      });
      const key = values.join("|");
      let index = styleIndex.get(key);
      if (index === undefined) {
        index = styles.length;
        styles.push(values);
        styleIndex.set(key, index);
      }
      return index;
    }

    // Clip is the visible area an ancestor with overflow other than visible
    // leaves. Carousel items and tab panels beyond it are laid out but never seen.
    function intersect(a, b) {
      if (!a) return b;
      const left = Math.max(a.left, b.left);
      const top = Math.max(a.top, b.top);
      const right = Math.min(a.right, b.right);
      const bottom = Math.min(a.bottom, b.bottom);
      return { left, top, right: Math.max(left, right), bottom: Math.max(top, bottom) };
    }

    function visit(element, parentIndex, clip) {
      const tag = element.tagName.toLowerCase();
      if (SKIP_TAGS.has(tag)) {
        hidden.add(element);
        return;
      }
      const computed = getComputedStyle(element);
      if (
        computed.display === "none" ||
        computed.visibility === "hidden" ||
        computed.opacity === "0" ||
        computed.contentVisibility === "hidden"
      ) {
        hidden.add(element);
        return;
      }

      const rect = element.getBoundingClientRect();
      const fixed = computed.position === "fixed";
      // A fixed box escapes the clipping of its ancestors.
      const ownClip = fixed ? null : clip;
      const visible = ownClip ? intersect(ownClip, rect) : rect;
      const visibleArea = (visible.right - visible.left) * (visible.bottom - visible.top);
      if (ownClip && rect.width * rect.height > 0 && visibleArea <= 0) {
        hidden.add(element);
        return;
      }
      const inline = computed.display === "inline" || computed.display === "contents";
      const leaf = LEAF_TAGS.has(tag);
      const offscreen =
        rect.right <= 0 || rect.left >= viewportWidth || rect.bottom + (fixed ? 0 : scrollY) <= 0;
      const significant =
        !offscreen &&
        (!inline || leaf) &&
        rect.width >= MIN_SIDE &&
        rect.height >= MIN_SIDE &&
        rect.width * rect.height >= MIN_AREA;

      let index = parentIndex;
      if (significant) {
        index = nodes.length;
        const offsetX = fixed ? 0 : scrollX;
        const offsetY = fixed ? 0 : scrollY;
        let clipped = null;
        const area = rect.width * rect.height;
        if (ownClip && area > 0 && visibleArea < area * 0.98) {
          clipped = [
            Math.round(visible.left + offsetX),
            Math.round(visible.top + offsetY),
            Math.round(visible.right - visible.left),
            Math.round(visible.bottom - visible.top),
          ];
        }
        const testid = element.getAttribute("data-testid") ?? element.getAttribute("data-test-id");
        const image = tag === "img" ? element.currentSrc || element.src || "" : null;
        nodes.push([
          parentIndex,
          tag,
          element.getAttribute("class") ?? "",
          testid,
          Math.round(rect.left + offsetX),
          Math.round(rect.top + offsetY),
          Math.round(rect.width),
          Math.round(rect.height),
          styleOf(computed),
          0,
          image === null ? null : fnv32(image),
          fixed ? FLAG_FIXED : 0,
          clipped,
        ]);
        texts.push("");
        owner.set(element, index);
      } else if (offscreen && !inline) {
        // Off-canvas menus and carousel slides past the edge. Their text
        // must not land on an ancestor either.
        hidden.add(element);
        return;
      }

      if (leaf) return;
      const clips = computed.overflowX !== "visible" || computed.overflowY !== "visible";
      const childClip = clips ? intersect(ownClip, rect) : ownClip;
      for (const child of element.children) visit(child, index, childClip);
    }

    if (document.body) visit(document.body, -1, null);

    const walker = document.createTreeWalker(document.body ?? document, NodeFilter.SHOW_TEXT);
    while (walker.nextNode()) {
      const text = walker.currentNode.data.replace(/\s+/g, " ").trim();
      if (!text) continue;
      let element = walker.currentNode.parentElement;
      let dropped = false;
      while (element && !owner.has(element)) {
        if (hidden.has(element)) {
          dropped = true;
          break;
        }
        element = element.parentElement;
      }
      if (dropped || !element || hidden.has(element)) continue;
      const index = owner.get(element);
      if (texts[index].length < 4000) texts[index] += (texts[index] ? " " : "") + text;
    }
    texts.forEach((text, index) => {
      if (text) nodes[index][9] = fnv32(text);
    });

    return {
      props: STYLE_PROPS,
      types: structuredDataTypes(),
      width: viewportWidth,
      height: document.documentElement.scrollHeight,
      styles,
      nodes,
    };
  }

  function restore() {
    changed.style?.remove();
    changed.style = null;
    for (const image of changed.images) image.loading = "lazy";
    changed.images = [];
  }

  function stash() {
    // Escaping "<" keeps "</script>" inside the JSON from ending the element.
    const json = JSON.stringify(capture()).replace(/</g, "\\u003c");
    restore();
    document.getElementById(STASH_ID)?.remove();
    const element = document.createElement("script");
    element.id = STASH_ID;
    element.type = "application/json";
    element.textContent = json;
    (document.head || document.documentElement).append(element);
  }

  const api = { prepared: false, prepare, capture, stash };
  window.__shoutingRobinLayout = api;
})();

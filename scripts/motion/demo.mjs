// Plays the motion demo of the website: the test site typed into the crawl
// bar and crawled in Chrome while its pages arrive, the findings on the
// Overview, a page's details, and the accessibility violations axe-core found.
// Every state shown is a keyframe Shouting Robin exported (the `motion` scene
// in crates/shoutingrobin/src/screenshots.rs). What moves between them is
// drawn here: the camera, the typing, the caret and the pointer. It plays once
// and stops on the whole window, with hotspots over features that tell more
// about them when pointed at.
//
//   import { mountDemo } from "./demo.mjs";
//   const bundle = await (await fetch("demo.json")).json();
//   const demo = mountDemo(element, bundle);
//
// A frame is a function of time alone, so `demo.seek(t)` shows exactly what
// playback shows at `t` seconds, as a video capture needs.

import { renderSvg } from "./render.mjs";

/// What the scene types into the crawl bar, as the keyframe `02-url` holds it.
const URL_TEXT = "http://127.0.0.1:8000/";

const CARET = "#f5e0dc";

/// Keyframes captured after the demo, only for the hotspots' cards.
const MEDIA_KEYS = new Set(["12-performance", "13-ecommerce", "14-changes"]);

/// The hotspots shown once the demo stops: what each says, and how to find
/// where it goes in the last keyframe. The copy follows site/index.html. A
/// card with `media` shows that crop of the keyframe named by `key`.
const HOTSPOTS = [
  {
    title: "Chrome rendering",
    text: "Turn on Chrome to crawl pages after JavaScript has run. The server HTML is fetched as well, and Chrome crawls add LCP, CLS, FCP and TTFB.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Chrome" && r.y < 70)),
    key: "12-performance",
    media: (keyframes) => gridCrop(keyframes["12-performance"].items, ["ADDRESS", "CLS"], 7),
  },
  {
    title: "History",
    text: "Every crawl is saved in a local SQLite database. Open, recrawl or delete earlier crawls from the sidebar.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Historic Crawls")),
  },
  {
    title: "Changes",
    text: "A crawl is compared with the previous crawl of the same URL: pages added or removed, and pages whose status, indexability, title, H1, meta description or word count changed.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Changes" && r.y < 120)),
    key: "14-changes",
    media: (keyframes) => gridCrop(keyframes["14-changes"].items, ["ADDRESS", "DETAIL"], 7, 900),
  },
  {
    title: "Over 90 checks",
    text: "Each check is a filter on its tab, so the URLs it flags are one click away.",
    find: (items) => anchorOn(textRun(items, (r) => r.text.startsWith("image-alt") && r.y < 150)),
  },
  {
    title: "Reports",
    text: "Export the Overview as a PDF report with sample URLs for each finding, the current tab as CSV, or every tab as CSV files at once.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Export All")),
  },
  {
    title: "Page details",
    text: "The status and indexability of the page, its title, description and headings with their lengths, and a preview of the search result.",
    find: (items) => anchorAfter(textRun(items, (r) => r.text === "SERP PREVIEW")),
  },
  {
    title: "Accessibility",
    text: "Chrome crawls run axe-core on every page. Each violation is a row with its rule, impact, the element it found and its HTML.",
    find: (items) => anchorOn(topmost(items, (r) => r.text === "critical")),
  },
  {
    title: "Ecommerce",
    text: "Product pages get their own checks: a missing price, availability, SKU, GTIN, brand or image, GTINs with the wrong check digit, and indexable pages that are out of stock.",
    find: (items) => anchorAfter(textRun(items, (r) => r.text === "/about.html")),
    key: "13-ecommerce",
    media: (keyframes) => gridCrop(keyframes["13-ecommerce"].items, ["ADDRESS", "SKU"], 7),
  },
];

export function mountDemo(root, bundle, options = {}) {
  const { keyframes, icons } = bundle;
  // The window as the keyframes hold it, in logical pixels.
  const { width, height } = keyframes["01-start"];
  const reduced =
    options.reducedMotion ??
    globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

  injectStyles();
  root.classList.add("bd-root");
  root.style.setProperty("--bd-width", width);
  root.style.setProperty("--bd-height", height);
  root.innerHTML = `
    <div class="bd-viewport">
      <div class="bd-stage"><div class="bd-camera"></div></div>
      <div class="bd-overlay">
        <div class="bd-caret"></div>
        <div class="bd-ring"></div>
        <div class="bd-ripple"></div>
        <svg class="bd-pointer" viewBox="0 0 16 22" aria-hidden="true">
          <path d="M1 1v17l4.5-4.2 3 6.7 3-1.4-3-6.6H15z" fill="#fff" stroke="#11111b" stroke-width="1.4" stroke-linejoin="round"/>
        </svg>
        <div class="bd-keys"></div>
        <div class="bd-caption"></div>
        <div class="bd-dim"></div>
        <button class="bd-replay" type="button">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
          Replay
        </button>
      </div>
    </div>
    <div class="bd-spots"></div>`;
  const viewport = root.querySelector(".bd-viewport");
  const stage = root.querySelector(".bd-stage");
  const camera = root.querySelector(".bd-camera");
  const caret = root.querySelector(".bd-caret");
  const ring = root.querySelector(".bd-ring");
  const ripple = root.querySelector(".bd-ripple");
  const pointer = root.querySelector(".bd-pointer");
  const keys = root.querySelector(".bd-keys");
  const caption = root.querySelector(".bd-caption");
  const dim = root.querySelector(".bd-dim");
  const spots = root.querySelector(".bd-spots");
  const replay = root.querySelector(".bd-replay");

  const layers = {};
  const page = Math.random().toString(36).slice(2, 6);
  const timeline = Object.entries(keyframes).filter(([name]) => !MEDIA_KEYS.has(name));
  for (const [ix, [name, snapshot]] of timeline.entries()) {
    const layer = document.createElement("div");
    layer.className = "bd-layer";
    // Every layer is on the same page, so each needs ids of its own.
    const idPrefix = `bd${page}k${ix}-`;
    layer.innerHTML = renderSvg(snapshot, { icons, fonts: options.fonts, idPrefix });
    camera.append(layer);
    layers[name] = layer;
  }

  // The crawl bar as it is once typing starts: the address keyframe without
  // its text. It covers the placeholder where the address is still to come.
  const cover = document.createElement("div");
  cover.className = "bd-layer";
  cover.style.zIndex = 2;
  cover.innerHTML = renderSvg(
    { ...keyframes["02-url"], items: keyframes["02-url"].items.filter((item) => item.t !== "text") },
    { icons, idPrefix: `bd${page}c-` },
  );
  camera.append(cover);

  const plan = makePlan(keyframes, { width, height });
  const rows = rowElements(layers[plan.staggerKey], keyframes[plan.staggerKey], plan.grid);
  buildHotspots(spots, dim, keyframes, plan.lastKey, {
    docs: options.docs ?? null,
    size: { width, height },
    scale: () => scale,
    // A card's crop of a keyframe is an SVG of its own, with ids of its own.
    media: (key, box, ix) =>
      renderSvg(keyframes[key], {
        icons,
        fonts: options.fonts,
        viewBox: box,
        idPrefix: `bd${page}m${ix}-`,
      }),
  });

  let scale = 1;
  const resize = () => {
    scale = viewport.clientWidth / width;
    stage.style.transform = `scale(${scale})`;
  };
  new ResizeObserver(resize).observe(viewport);
  resize();

  let shownCaption = null;
  let shownKeys = null;
  let staggered = false;

  function frame(t) {
    if (reduced) t = plan.duration;
    t = Math.min(t, plan.duration);
    // Stopped on the whole window, the hotspots take over.
    const ended = t >= plan.duration;
    root.classList.toggle("bd-ended", ended);

    // The camera, as the point of the stage at the centre of the view and a
    // zoom.
    const view = interpolate(plan.camera, t);
    const zoom = view.z;
    const tx = clamp(width / 2 - view.x * zoom, width - width * zoom, 0);
    const ty = clamp(height / 2 - view.y * zoom, height - height * zoom, 0);
    camera.style.transform = `translate(${tx}px, ${ty}px) scale(${zoom})`;
    const toScreen = (x, y) => [(x * zoom + tx) * scale, (y * zoom + ty) * scale];

    // Keyframes: the one shown, and the one under it while it fades in or
    // is revealed character by character.
    const { base, top, opacity, reveal } = layerState(plan, t);
    for (const [name, layer] of Object.entries(layers)) {
      const visible = name === base || name === top;
      layer.style.display = visible ? "" : "none";
      layer.style.zIndex = name === top ? 3 : 1;
      layer.style.opacity = name === top ? opacity : 1;
      layer.style.clipPath = name === top && reveal ? notch(reveal, width, height) : "";
    }
    cover.style.display = reveal?.cover ? "" : "none";
    cover.style.clipPath = reveal?.cover ? hole(reveal) : "";

    // The first pages of the crawl come in one after another.
    const stagger = t - plan.staggerAt;
    if (base === plan.staggerKey || top === plan.staggerKey) {
      rows.forEach((elements, row) => {
        const p = easeOut(clamp((stagger - 0.05 - row * 0.09) / 0.3, 0, 1));
        for (const element of elements) {
          element.style.opacity = p;
          element.style.transform = `translateY(${(1 - p) * 8}px)`;
        }
      });
      staggered = true;
    } else if (staggered) {
      rows.flat().forEach((element) => {
        element.style.opacity = "";
        element.style.transform = "";
      });
      staggered = false;
    }

    // The caret, solid while typing and blinking while it waits.
    const caretAt = caretState(plan, t);
    if (caretAt) {
      const [x, y] = toScreen(caretAt.x, caretAt.top);
      caret.style.display = "";
      caret.style.transform = `translate(${x}px, ${y}px)`;
      caret.style.height = `${caretAt.height * zoom * scale}px`;
      caret.style.width = `${Math.max(1.5, 1.6 * zoom * scale)}px`;
      caret.style.opacity = caretAt.blink ? (Math.floor(t * 1.8) % 2 ? 0 : 1) : 1;
    } else {
      caret.style.display = "none";
    }

    // The pointer, and the ripples of its clicks.
    const hand = interpolate(plan.pointer, t);
    if (hand.o > 0.01) {
      const [x, y] = toScreen(hand.x, hand.y);
      pointer.style.display = "";
      pointer.style.opacity = hand.o;
      pointer.style.transform = `translate(${x - 2}px, ${y - 1}px)`;
    } else {
      pointer.style.display = "none";
    }
    const click = plan.clicks.findLast((c) => t >= c.t && t < c.t + 0.45);
    if (click) {
      const p = (t - click.t) / 0.45;
      const [x, y] = toScreen(click.x, click.y);
      const size = 10 + 34 * easeOut(p);
      ripple.style.display = "";
      ripple.style.opacity = 0.55 * (1 - p);
      ripple.style.width = ripple.style.height = `${size}px`;
      ripple.style.transform = `translate(${x - size / 2}px, ${y - size / 2}px)`;
    } else {
      ripple.style.display = "none";
    }

    // A ring drawn around what the scene points at.
    const highlight = plan.rings.find((r) => t >= r.from && t < r.to);
    if (highlight) {
      // Kept inside the view, which a button at the window's edge would
      // otherwise push the ring out of.
      const [x0, y0] = toScreen(highlight.b[0] - 4, highlight.b[1] - 4).map((v) => Math.max(v, 3));
      const [x1, y1] = toScreen(highlight.b[0] + highlight.b[2] + 4, highlight.b[1] + highlight.b[3] + 4).map(
        (v, ix) => Math.min(v, (ix ? height : width) * scale - 3),
      );
      const p = clamp(Math.min(t - highlight.from, highlight.to - t) / 0.3, 0, 1);
      ring.style.display = "";
      ring.style.opacity = p;
      ring.style.transform = `translate(${x0}px, ${y0}px)`;
      ring.style.width = `${x1 - x0}px`;
      ring.style.height = `${y1 - y0}px`;
    } else {
      ring.style.display = "none";
    }

    // Keys pressed, shown as keycaps.
    const press = plan.keys.find((k) => t >= k.from && t < k.to);
    if (press !== shownKeys) {
      keys.innerHTML = press ? press.keys.map((key) => `<kbd>${key}</kbd>`).join("") : "";
      shownKeys = press;
    }
    if (press) {
      const p = clamp(Math.min(t - press.from, press.to - t) / 0.18, 0, 1);
      keys.style.opacity = p;
      keys.classList.toggle("bd-pressed", t >= press.down && t < press.down + 0.18);
      keys.style.transform = `translate(-50%, ${(1 - p) * 10}px)`;
    }

    const words = ended
      ? { text: "Point at a dot to see more", from: -Infinity, to: Infinity }
      : plan.captions.find((c) => t >= c.from && t < c.to);
    if (words !== shownCaption) {
      caption.textContent = words ? words.text : "";
      shownCaption = words;
    }
    caption.style.opacity = ended
      ? ""
      : words
        ? clamp(Math.min(t - words.from, words.to - t) / 0.35, 0, 1)
        : 0;
  }

  let start = null;
  let offset = 0;
  let playing = false;
  let raf = 0;
  const tick = (now) => {
    if (start === null) start = now - offset * 1000;
    const t = (now - start) / 1000;
    offset = Math.min(t, plan.duration);
    frame(offset);
    if (t >= plan.duration) {
      playing = false;
      return;
    }
    raf = requestAnimationFrame(tick);
  };
  const play = () => {
    if (playing || reduced) return;
    if (offset >= plan.duration) offset = 0;
    playing = true;
    start = null;
    raf = requestAnimationFrame(tick);
  };
  replay.addEventListener("click", () => {
    pause();
    offset = 0;
    play();
  });
  const pause = () => {
    playing = false;
    cancelAnimationFrame(raf);
  };

  frame(0);
  if (options.autoplay !== false) {
    // Only play while on screen, and only once: scrolled back to, it stays
    // stopped on the hotspots.
    new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) pause();
        else if (offset < plan.duration) play();
      },
      { threshold: 0.2 },
    ).observe(viewport);
  }

  return {
    duration: plan.duration,
    get ended() {
      return offset >= plan.duration;
    },
    play,
    pause,
    seek(t) {
      pause();
      offset = t;
      frame(t);
    },
  };
}

/// Adds the hotspots, placed by where their features are in `items`, the last
/// keyframe's. Pointing at one, or focusing or tapping it, shows its card and
/// dims the rest of the window around it.
function buildHotspots(container, dim, keyframes, lastKey, { docs, size, scale, media }) {
  const { width, height } = size;
  const items = keyframes[lastKey].items;
  const base = docs && (docs.endsWith("/") ? docs : `${docs}/`);
  let index = 0;
  for (const spot of HOTSPOTS) {
    let at;
    try {
      at = spot.find(items);
    } catch {
      continue;
    }
    let figure = "";
    if (spot.media) {
      try {
        const box = spot.media(keyframes);
        const key = spot.key;
        figure = `<div class="bd-media" style="aspect-ratio: ${box[2]} / ${box[3]}">${media(key, box, index)}</div>`;
      } catch {
        // Without the keyframe the card goes without its picture.
      }
    }
    const id = `bd-card-${Math.random().toString(36).slice(2, 8)}`;
    const element = document.createElement("div");
    element.className = "bd-spot";
    element.style.left = `${(at.x / width) * 100}%`;
    element.style.top = `${(at.y / height) * 100}%`;
    element.style.setProperty("--i", index++);
    element.dataset.side = at.x > width * 0.55 ? "left" : "right";
    element.dataset.rise = at.y > height * 0.55 ? "up" : "down";
    element.innerHTML = `
      <button class="bd-dot" type="button" aria-label="${spot.title}" aria-describedby="${id}"></button>
      <div class="bd-card${figure ? " bd-wide" : ""}" role="tooltip" id="${id}">
        ${figure}
        <strong>${spot.title}</strong>
        <p>${spot.text}</p>
        ${base && spot.page ? `<a href="${base}${spot.page}">Read more</a>` : ""}
      </div>`;
    const show = () => {
      for (const other of container.querySelectorAll(".bd-open")) {
        if (other !== element) other.classList.remove("bd-open");
      }
      element.classList.add("bd-open");
      const radius = 70 * scale();
      dim.style.background = `radial-gradient(circle at ${(at.x / width) * 100}% ${(at.y / height) * 100}%, transparent ${radius}px, rgb(17 17 27 / 0.62) ${radius + 70 * scale()}px)`;
      dim.classList.add("bd-on");
    };
    const hide = () => {
      element.classList.remove("bd-open");
      if (!container.querySelector(".bd-open")) dim.classList.remove("bd-on");
    };
    element.addEventListener("pointerenter", show);
    element.addEventListener("pointerleave", hide);
    element.addEventListener("focusin", show);
    element.addEventListener("focusout", (event) => {
      if (!element.contains(event.relatedTarget)) hide();
    });
    // A tap opens a card, a second tap closes it.
    element.querySelector(".bd-dot").addEventListener("click", () =>
      element.classList.contains("bd-open") ? hide() : show(),
    );
    container.append(element);
  }
}

function textRun(items, test) {
  const run = items.find((item) => item.t === "text" && test(item));
  if (!run) throw new Error("no such text");
  return run;
}

/// The box a text run fills: its clip when that is a button or badge around
/// it, otherwise the text itself.
function runBox(run) {
  const right = run.x[run.x.length - 1] + run.size * 0.6;
  if (run.clip && run.clip[2] < 220) return run.clip;
  return [run.x[0], run.y - run.size, right - run.x[0], run.size * 1.3];
}

/// A hotspot just off the top right corner of what a run labels, as a badge
/// sits, clear of the label itself.
function anchorOn(run) {
  const [x, y, w] = runBox(run);
  return { x: x + w + 3, y: y - 1 };
}

function union(boxes) {
  const left = Math.min(...boxes.map((b) => b[0]));
  const top = Math.min(...boxes.map((b) => b[1]));
  const right = Math.max(...boxes.map((b) => b[0] + b[2]));
  const bottom = Math.max(...boxes.map((b) => b[1] + b[3]));
  return [left, top, right - left, bottom - top];
}

function pad([x, y, w, h], by) {
  return [x - by, y - by, w + by * 2, h + by * 2];
}

/// A hotspot just after the end of a run's text, on its line.
function anchorAfter(run) {
  return { x: run.x[run.x.length - 1] + run.size * 1.8, y: run.y - run.size * 0.35 };
}

/// The topmost text run that passes `test`. Runs come in painting order, so
/// the first found need not be the first row.
function topmost(items, test) {
  const runs = items.filter((item) => item.t === "text" && test(item));
  if (!runs.length) throw new Error("no such text");
  return runs.reduce((top, run) => (run.y < top.y ? run : top));
}

/// A crop of a grid: its header from the `first` heading to a little past the
/// `last` one, or `width` wide, and `rows` rows below it.
function gridCrop(items, [first, last], rows, width) {
  const left = textRun(items, (r) => r.text === first);
  const right = textRun(items, (r) => r.text === last);
  const x = left.x[0] - 8;
  const end = width ? x + width : right.x[right.x.length - 1] + right.size * 6;
  const top = left.y - 20;
  return [x, top, end - x, 32 + rows * 32];
}

/// Works out the timeline from where things are in the keyframes.
function makePlan(keyframes, size) {
  const { width, height } = size;
  // Typing: the address, and when each character appears.
  // The crawl bar's, the topmost: the history lists a crawl by the same
  // address.
  const url = topmost(keyframes["02-url"].items, (r) => r.text === URL_TEXT);
  const typingStart = 1.7;
  let at = typingStart;
  let seed = 11;
  const random = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
  const chars = [...URL_TEXT].map((_, ix) => {
    at += 0.04 + random() * 0.035;
    return { t: at, x: url.x[ix] };
  });
  const typed = at + 0.1;
  const line = {
    chars,
    top: url.y - url.size * 1.05,
    bottom: url.y + url.size * 0.4,
    // What is still to be typed: the glyphs and no more.
    hidden: {
      top: url.y - url.size * 0.95,
      bottom: url.y + url.size * 0.3,
      right: chars[chars.length - 1].x + url.size * 0.7,
    },
  };

  const play = keyframes["02-url"].items.find(
    (item) => item.t === "icon" && item.path.includes("play"),
  );
  const playPoint = [play.b[0] + play.b[2] / 2, play.b[1] + play.b[3] / 2];
  const chrome = findRun(keyframes["02-url"], (r) => r.text === "Chrome");
  const crawl = typed + 1.3;
  const crawling = ["03-crawling-1", "04-crawling-2", "05-crawling-3", "06-crawled"].map(
    (key, ix) => ({ key, t: crawl + ix * 1.15 }),
  );
  const crawled = crawling[crawling.length - 1].t;

  const firstRow = topmost(
    keyframes["03-crawling-1"].items,
    (r) => r.text.startsWith("/") && r.x[0] < 300,
  );
  const grid = { top: firstRow.y - 21, rowHeight: 32, left: 250, bottom: 950 };

  const tab = (key, label) => {
    const run = findRun(keyframes[key], (r) => r.text === label && r.y < 120);
    return [run.clip[0] + run.clip[2] / 2, run.clip[1] + run.clip[3] / 2];
  };
  const overviewTab = tab("06-crawled", "Overview");
  const titlesTab = tab("07-overview", "Page Titles");
  const overview = crawled + 1.0;
  const titles = overview + 2.4;
  const detailsRow = findRun(keyframes["08-titles"], (r) => r.text === "/redirect-target.html");
  const detailsPoint = [detailsRow.x[0] + 70, detailsRow.y - 4];
  const details = titles + 1.1;
  const accessibility = details + 2.6;
  const violationRow = topmost(keyframes["10-accessibility"].items, (r) => r.text === "/a11y.html");
  const violationPoint = [violationRow.x[0] + 40, violationRow.y - 4];
  const violation = accessibility + 1.4;
  const outro = violation + 1.8;
  const duration = outro + 1.4;
  // The Chrome switch and its label, for the ring while the crawl starts.
  const chromeBox = [chrome.x[0] - 40, chrome.y - 14, chrome.x[chrome.x.length - 1] - chrome.x[0] + 52, 20];

  const view = (x, y, z) => ({ x, y, z });
  return {
    duration,
    lastKey: "11-violation",
    staggerKey: "03-crawling-1",
    staggerAt: crawl,
    grid,
    line,
    typingStart,
    typed,
    crawl,
    crawling,
    overview,
    titles,
    details,
    accessibility,
    violation,
    width,
    camera: [
      { t: 0, ...view(width / 2, height / 2, 1) },
      { t: 0.6, ...view(width / 2, height / 2, 1) },
      // Onto the crawl bar for the typing.
      { t: 1.6, ...view(420, 120, 2.1) },
      { t: typed + 0.1, ...view(440, 120, 2.1) },
      // Along to the Chrome switch and the crawl button.
      { t: crawl - 0.2, ...view(1380, 140, 2.0) },
      // Out to the grid as the pages come in.
      { t: crawl + 1.1, ...view(720, 380, 1.45) },
      { t: crawled, ...view(720, 420, 1.45) },
      // The Overview's findings.
      { t: overview + 0.5, ...view(760, 330, 1.5) },
      { t: titles - 0.4, ...view(800, 330, 1.5) },
      // Page Titles, the page picked, then its details.
      { t: details - 0.3, ...view(700, 560, 1.45) },
      { t: details + 0.8, ...view(1290, 520, 1.6) },
      { t: accessibility - 0.2, ...view(1290, 560, 1.6) },
      // The violations.
      { t: accessibility + 0.8, ...view(700, 330, 1.5) },
      { t: violation + 0.6, ...view(760, 340, 1.5) },
      { t: outro, ...view(780, 360, 1.45) },
      { t: outro + 1.3, ...view(width / 2, height / 2, 1) },
      { t: duration, ...view(width / 2, height / 2, 1) },
    ],
    pointer: [
      { t: 0, x: 1200, y: 400, o: 0 },
      { t: typed + 0.2, x: 1300, y: 200, o: 0 },
      { t: typed + 0.5, x: 1300, y: 200, o: 1 },
      { t: crawl, x: playPoint[0], y: playPoint[1], o: 1 },
      { t: crawl + 0.4, x: playPoint[0], y: playPoint[1], o: 1 },
      { t: crawl + 1.1, x: 1100, y: 520, o: 1 },
      { t: overview - 0.6, x: 1100, y: 520, o: 1 },
      { t: overview, x: overviewTab[0], y: overviewTab[1], o: 1 },
      { t: titles - 0.7, x: overviewTab[0] + 40, y: overviewTab[1] + 60, o: 1 },
      { t: titles, x: titlesTab[0], y: titlesTab[1], o: 1 },
      { t: details, x: detailsPoint[0], y: detailsPoint[1], o: 1 },
      { t: accessibility - 0.6, x: detailsPoint[0] + 30, y: detailsPoint[1] + 40, o: 1 },
      { t: accessibility, x: detailsPoint[0] + 30, y: detailsPoint[1] + 40, o: 0 },
      { t: accessibility + 0.5, x: violationPoint[0] + 80, y: violationPoint[1] + 120, o: 1 },
      { t: violation, x: violationPoint[0], y: violationPoint[1], o: 1 },
      { t: outro, x: violationPoint[0], y: violationPoint[1], o: 1 },
      { t: outro + 0.5, x: violationPoint[0], y: violationPoint[1], o: 0 },
      { t: duration, x: 1200, y: 400, o: 0 },
    ],
    clicks: [
      { t: crawl, x: playPoint[0], y: playPoint[1] },
      { t: overview, x: overviewTab[0], y: overviewTab[1] },
      { t: titles, x: titlesTab[0], y: titlesTab[1] },
      { t: details, x: detailsPoint[0], y: detailsPoint[1] },
      { t: violation, x: violationPoint[0], y: violationPoint[1] },
    ],
    rings: [{ from: typed + 0.4, to: crawl + 0.3, b: chromeBox }],
    keys: [],
    captions: [
      { from: 0.8, to: typed + 0.3, text: "Crawl a site from your machine" },
      { from: typed + 0.5, to: crawled - 0.1, text: "Chrome renders every page after its JavaScript has run" },
      { from: overview - 0.1, to: titles - 0.1, text: "Findings are grouped and sorted by priority" },
      { from: titles + 0.1, to: accessibility - 0.1, text: "Select a row to see the page beside the grid" },
      { from: accessibility + 0.1, to: outro + 0.6, text: "axe-core audits every page for accessibility" },
    ],
  };
}

/// Which keyframes show at `t`.
function layerState(plan, t) {
  if (t < plan.typingStart) return { base: "01-start" };
  if (t < plan.typed) {
    const shown = plan.line.chars.filter((c) => c.t <= t).length;
    const x = shown < plan.line.chars.length ? plan.line.chars[shown].x : plan.width;
    return {
      base: "01-start",
      top: "02-url",
      opacity: 1,
      reveal: { x, ...plan.line.hidden, cover: true },
    };
  }
  if (t < plan.crawl) return { base: "02-url" };
  if (t < plan.overview) {
    for (let ix = plan.crawling.length - 1; ix >= 0; ix--) {
      const step = plan.crawling[ix];
      if (t >= step.t) {
        const below = ix === 0 ? "02-url" : plan.crawling[ix - 1].key;
        return fade(below, step.key, (t - step.t) / 0.25);
      }
    }
  }
  if (t < plan.titles) return fade("06-crawled", "07-overview", (t - plan.overview) / 0.2);
  if (t < plan.details) return fade("07-overview", "08-titles", (t - plan.titles) / 0.2);
  if (t < plan.accessibility) return fade("08-titles", "09-details", (t - plan.details) / 0.2);
  if (t < plan.violation) {
    return fade("09-details", "10-accessibility", (t - plan.accessibility) / 0.3);
  }
  return fade("10-accessibility", "11-violation", (t - plan.violation) / 0.2);
}

/// Where the caret is at `t`, if it shows: in the crawl bar, before and while
/// the address is typed.
function caretState(plan, t) {
  const { line } = plan;
  if (t < 0.9 || t >= plan.typed + 0.6) return null;
  const shown = line.chars.filter((c) => c.t <= t).length;
  const step = line.chars[1].x - line.chars[0].x;
  const x = shown === 0 ? line.chars[0].x : line.chars[shown - 1].x + step;
  return {
    x,
    top: line.top + 1,
    height: line.bottom - line.top,
    blink: t < plan.typingStart || t >= plan.typed,
  };
}

/// A clip path showing only the hole a reveal leaves, where the typing has
/// not reached yet.
function hole({ x, top, bottom, right }) {
  if (x >= right) return "inset(100%)";
  return `polygon(${x}px ${top}px, ${right}px ${top}px, ${right}px ${bottom}px, ${x}px ${bottom}px)`;
}

function fade(base, top, p) {
  p = clamp(p, 0, 1);
  return p >= 1 ? { base: top } : { base, top, opacity: easeInOut(p) };
}


/// The elements of the result's rows, by row, so they can come in in turn.
function rowElements(layer, snapshot, grid) {
  const rows = [];
  for (const element of layer.querySelectorAll("[data-i]")) {
    const item = snapshot.items[Number(element.dataset.i)];
    const box = item.b ?? (item.t === "text" ? [item.x[0], item.y - item.size, 0, item.size] : null);
    if (!box || box[3] > grid.rowHeight + 2) continue;
    const middle = box[1] + box[3] / 2;
    if (middle < grid.top || middle > grid.bottom || box[0] < grid.left) continue;
    const row = Math.floor((middle - grid.top) / grid.rowHeight);
    (rows[row] ??= []).push(element);
  }
  return rows.map((row) => row ?? []);
}

function findRun(snapshot, test) {
  const run = snapshot.items.find((item) => item.t === "text" && test(item));
  if (!run) throw new Error("The keyframes do not hold the text the demo looks for");
  return run;
}

/// A clip path showing all of a layer but a hole from `x` to `right` on a
/// line, the text still to be typed.
function notch({ x, top, bottom, right }, width, height) {
  if (x >= right) return "";
  return `polygon(evenodd, 0 0, ${width}px 0, ${width}px ${height}px, 0 ${height}px, 0 0, ${x}px ${top}px, ${right}px ${top}px, ${right}px ${bottom}px, ${x}px ${bottom}px, ${x}px ${top}px)`;
}

/// The value of keyed `frames` at `t`, eased between keys.
function interpolate(frames, t) {
  if (t <= frames[0].t) return frames[0];
  for (let ix = 1; ix < frames.length; ix++) {
    const a = frames[ix - 1];
    const b = frames[ix];
    if (t < b.t) {
      const p = easeInOut((t - a.t) / (b.t - a.t));
      const out = {};
      for (const key of Object.keys(b)) out[key] = a[key] + (b[key] - a[key]) * p;
      return out;
    }
  }
  return frames[frames.length - 1];
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

function easeInOut(p) {
  return p < 0.5 ? 4 * p * p * p : 1 - (-2 * p + 2) ** 3 / 2;
}

function easeOut(p) {
  return 1 - (1 - p) ** 3;
}

let styled = false;
function injectStyles() {
  if (styled) return;
  styled = true;
  const style = document.createElement("style");
  style.textContent = `
    .bd-root { container-type: inline-size; position: relative; }
    .bd-viewport {
      position: relative; aspect-ratio: var(--bd-width) / var(--bd-height); overflow: hidden;
      border-radius: 12px; background: #11111b;
      box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08), 0 40px 80px -30px rgb(0 0 0 / 0.6);
    }
    .bd-stage { position: absolute; left: 0; top: 0; width: calc(var(--bd-width) * 1px); height: calc(var(--bd-height) * 1px); transform-origin: 0 0; }
    .bd-camera { position: absolute; inset: 0; transform-origin: 0 0; will-change: transform; }
    .bd-layer { position: absolute; inset: 0; }
    .bd-layer svg { display: block; }
    .bd-layer [data-i] { transform-box: fill-box; }
    .bd-overlay { position: absolute; inset: 0; pointer-events: none; }
    .bd-overlay > * { position: absolute; left: 0; top: 0; }
    .bd-caret { background: ${CARET}; border-radius: 1px; }
    .bd-pointer { width: 18px; height: 25px; filter: drop-shadow(0 2px 3px rgb(0 0 0 / 0.4)); }
    .bd-ripple { border-radius: 50%; background: #cba6f7; }
    .bd-ring { border: 2px solid #cba6f7; border-radius: 8px; box-shadow: 0 0 0 4px rgb(203 166 247 / 0.18); }
    .bd-keys {
      left: 50%; top: auto; bottom: 18%; display: flex; gap: 8px;
    }
    .bd-keys kbd {
      font: 600 clamp(14px, 2.4cqw, 26px)/1 system-ui, -apple-system, sans-serif;
      min-width: 1.9em; padding: 0.45em 0.6em; text-align: center; color: #cdd6f4;
      background: #313244; border-radius: 0.35em;
      box-shadow: 0 0.18em 0 #181825, 0 10px 24px -8px rgb(0 0 0 / 0.6), inset 0 1px 0 rgb(255 255 255 / 0.08);
      transition: transform 80ms, box-shadow 80ms;
    }
    .bd-keys.bd-pressed kbd { transform: translateY(0.12em); box-shadow: 0 0.06em 0 #181825, 0 6px 16px -8px rgb(0 0 0 / 0.6); }
    .bd-caption {
      left: 50%; top: 4.5%; transform: translateX(-50%);
      font: 500 clamp(12px, 1.7cqw, 19px)/1.2 system-ui, -apple-system, sans-serif;
      color: #cdd6f4; white-space: nowrap; padding: 0.6em 1em; border-radius: 999px;
      background: rgb(17 17 27 / 0.82); backdrop-filter: blur(8px);
      box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08), 0 12px 28px -12px rgb(0 0 0 / 0.7);
    }
    .bd-caption:empty { display: none; }
    /* The hint shows for a while, then clears the tabs it covers. */
    .bd-ended .bd-caption { animation: bd-hint 5s 0.9s both; }

    .bd-dim { inset: 0; width: 100%; height: 100%; opacity: 0; transition: opacity 0.25s; }
    .bd-dim.bd-on { opacity: 1; }

    .bd-replay {
      left: auto; top: auto; right: 2.2%; bottom: 3.5%; display: none; align-items: center; gap: 0.45em;
      font: 500 clamp(11px, 1.3cqw, 15px)/1 system-ui, -apple-system, sans-serif; color: #cdd6f4;
      padding: 0.6em 0.9em; border: 0; border-radius: 999px; cursor: pointer; pointer-events: auto;
      background: rgb(49 50 68 / 0.9); box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08), 0 10px 24px -10px rgb(0 0 0 / 0.7);
    }
    .bd-replay svg { width: 1.1em; height: 1.1em; }
    .bd-replay:hover { background: #45475a; }
    .bd-ended .bd-replay { display: inline-flex; animation: bd-in 0.4s 1.2s both; }

    .bd-spots { position: absolute; inset: 0; pointer-events: none; }
    .bd-spot { position: absolute; display: none; z-index: 1; }
    .bd-spot.bd-open { z-index: 2; }
    .bd-ended .bd-spot { display: block; animation: bd-pop 0.45s calc(0.25s + var(--i) * 0.09s) both; }
    .bd-dot {
      position: absolute; width: 16px; height: 16px; margin: -8px 0 0 -8px; padding: 0;
      border: 0; border-radius: 50%; cursor: pointer; pointer-events: auto;
      background: radial-gradient(circle, #fff 0 22%, #cba6f7 32% 100%);
      box-shadow: 0 0 0 0 rgb(203 166 247 / 0.7), 0 0 14px 2px rgb(203 166 247 / 0.65);
      animation: bd-glow 2s calc(var(--i) * -0.3s) infinite;
    }
    .bd-dot:focus-visible { outline: 2px solid #f5e0dc; outline-offset: 3px; }
    .bd-open .bd-dot { animation: none; box-shadow: 0 0 0 5px rgb(203 166 247 / 0.3), 0 0 18px 4px rgb(203 166 247 / 0.8); }
    .bd-card {
      position: absolute; width: min(270px, 70vw); box-sizing: border-box; padding: 14px 16px;
      pointer-events: auto; visibility: hidden; opacity: 0; transition: opacity 0.18s, transform 0.18s, visibility 0.18s;
      font: 400 13.5px/1.45 system-ui, -apple-system, sans-serif; color: #bac2de;
      background: rgb(30 30 46 / 0.97); border-radius: 12px;
      box-shadow: 0 0 0 1px rgb(203 166 247 / 0.35), 0 24px 48px -16px rgb(0 0 0 / 0.75);
    }
    .bd-card.bd-wide { width: min(360px, 80vw); }
    .bd-media {
      margin: -4px -6px 12px; border-radius: 8px; overflow: hidden; background: #11111b;
      box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08);
    }
    .bd-media svg { display: block; width: 100%; height: 100%; }
    .bd-card strong { display: block; font-weight: 600; font-size: 14.5px; color: #cdd6f4; margin-bottom: 4px; }
    .bd-card p { margin: 0 0 8px; }
    .bd-card a { color: #cba6f7; font-weight: 500; text-decoration: none; }
    .bd-card a:hover { text-decoration: underline; }
    .bd-card a::after { content: " →"; }
    .bd-spot[data-side="right"] .bd-card { left: 16px; }
    .bd-spot[data-side="left"] .bd-card { right: 16px; }
    .bd-spot[data-rise="down"] .bd-card { top: -18px; }
    .bd-spot[data-rise="up"] .bd-card { bottom: -18px; }
    .bd-spot[data-side="right"] .bd-card { transform: translateX(-6px); }
    .bd-spot[data-side="left"] .bd-card { transform: translateX(6px); }
    .bd-open .bd-card { visibility: visible; opacity: 1; transform: none !important; }

    @keyframes bd-glow {
      0% { box-shadow: 0 0 0 0 rgb(203 166 247 / 0.65), 0 0 14px 2px rgb(203 166 247 / 0.6); }
      70% { box-shadow: 0 0 0 12px rgb(203 166 247 / 0), 0 0 14px 2px rgb(203 166 247 / 0.6); }
      100% { box-shadow: 0 0 0 0 rgb(203 166 247 / 0), 0 0 14px 2px rgb(203 166 247 / 0.6); }
    }
    @keyframes bd-pop { from { opacity: 0; transform: scale(0.4); } to { opacity: 1; transform: none; } }
    @keyframes bd-in { from { opacity: 0; } }
    @keyframes bd-hint { 0% { opacity: 0; } 10%, 80% { opacity: 0.9; } 100% { opacity: 0; } }
    @media (prefers-reduced-motion: reduce) {
      .bd-dot, .bd-ended .bd-spot, .bd-ended .bd-caption, .bd-ended .bd-replay { animation: none; }
    }
  `;
  document.head.append(style);
}

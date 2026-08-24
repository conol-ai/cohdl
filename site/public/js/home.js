(() => {
  "use strict";

  document.documentElement.classList.add("js");

  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  const finePointer = window.matchMedia("(hover: hover) and (pointer: fine)");
  const desktopNav = window.matchMedia("(min-width: 901px)");
  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];

  const header = $("#site-header");
  const navToggle = $(".nav-toggle");
  const mobileNav = $("#mobile-nav");
  const progress = $("[data-scroll-progress]");
  const hudSection = $("[data-hud-section]");
  const hudPosition = $("[data-hud-position]");
  const themeMeta = $('meta[name="theme-color"]');

  let canvasController = null;
  let caseDrawerController = null;

  function setMenu(open, returnFocus = false) {
    if (!header || !navToggle || !mobileNav) return;

    header.classList.toggle("nav-open", open);
    navToggle.setAttribute("aria-expanded", String(open));
    navToggle.setAttribute("aria-label", open ? "Close navigation" : "Open navigation");
    mobileNav.inert = !open && !desktopNav.matches;

    if (open) {
      $("a", mobileNav)?.focus({ preventScroll: true });
    } else if (returnFocus) {
      navToggle.focus({ preventScroll: true });
    }
  }

  function initNavigation() {
    if (!header || !navToggle || !mobileNav) return;

    mobileNav.inert = !desktopNav.matches;
    navToggle.addEventListener("click", () => {
      setMenu(navToggle.getAttribute("aria-expanded") !== "true");
    });

    mobileNav.addEventListener("click", (event) => {
      if (event.target.closest("a")) setMenu(false);
    });

    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && header.classList.contains("nav-open")) {
        setMenu(false, true);
      }
    });

    document.addEventListener("pointerdown", (event) => {
      if (header.classList.contains("nav-open") && !header.contains(event.target)) {
        setMenu(false);
      }
    });

    desktopNav.addEventListener("change", (event) => {
      if (event.matches) {
        setMenu(false);
        mobileNav.inert = false;
      } else {
        mobileNav.inert = true;
      }
    });
  }

  function elementAtViewportPoint(selector, x, y) {
    for (const element of document.elementsFromPoint(x, y)) {
      const match = element.matches(selector) ? element : element.closest(selector);
      if (match) return match;
    }
    return null;
  }

  function setTheme(theme) {
    if (!theme || document.body.dataset.theme === theme) return;
    document.body.dataset.theme = theme;
    themeMeta?.setAttribute("content", theme === "light" ? "#f6f8fa" : "#070b11");
    canvasController?.setTheme(theme);
  }

  function initPipelineStack() {
    const pipeline = $("[data-pipeline]");
    if (!pipeline) return;

    const stages = $$("[data-pipeline-stage]", pipeline);
    const links = stages.map((stage) => $("[data-pipeline-link]", stage));
    const panels = stages.map((stage) => $(".pipeline-step", stage));
    const markers = stages.map((stage) => stage.previousElementSibling);
    const validMarkers = markers.every((marker, index) => {
      if (!marker?.matches(".pipeline-stage__marker[id]")) return false;
      return links[index]?.getAttribute("href") === `#${marker.id}`;
    });
    if (!stages.length || links.some((link) => !link) || panels.some((panel) => !panel) || !validMarkers) return;

    const stackViewport = window.matchMedia(
      "(min-width: 1041px) and (min-height: 760px), (max-width: 1040px) and (orientation: portrait)",
    );
    let pipelineFrame = 0;
    let pipelineFitFrame = 0;
    let activeIndex = 0;
    let viewportWidth = window.innerWidth;

    function isStacked() {
      return pipeline.classList.contains("is-scroll-stack");
    }

    function setActive(index) {
      activeIndex = Math.max(0, Math.min(stages.length - 1, index));
      const stacked = isStacked();
      const focusWouldBeCovered = stacked && panels.some((panel, panelIndex) => (
        panelIndex < activeIndex && panel.contains(document.activeElement)
      ));

      stages.forEach((stage, stageIndex) => {
        const current = stageIndex === activeIndex;
        const past = stageIndex < activeIndex;

        stage.classList.toggle("is-current", current);
        stage.classList.toggle("is-past", past);
        if (current) links[stageIndex].setAttribute("aria-current", "step");
        else links[stageIndex].removeAttribute("aria-current");
      });

      if (focusWouldBeCovered) links[activeIndex].focus({ preventScroll: true });
      panels.forEach((panel, panelIndex) => {
        panel.toggleAttribute("inert", stacked && panelIndex < activeIndex);
      });
    }

    function updatePipeline() {
      pipelineFrame = 0;
      const stacked = isStacked();
      const stickyTop = stacked ? Number.parseFloat(getComputedStyle(stages[0]).top) || 84 : 0;
      const probe = stacked ? stickyTop + 2 : Math.min(window.innerHeight * 0.36, 260);
      let nextIndex = 0;

      markers.forEach((marker, stageIndex) => {
        if (marker.getBoundingClientRect().top <= probe) nextIndex = stageIndex;
      });

      setActive(nextIndex);
    }

    function requestPipelineUpdate() {
      if (!pipelineFrame) pipelineFrame = requestAnimationFrame(updatePipeline);
    }

    function stackFitsViewport() {
      if (!isStacked()) return false;
      const stickyTop = Number.parseFloat(getComputedStyle(stages[0]).top) || 84;
      const visualHeight = window.visualViewport?.height || window.innerHeight;
      const viewportHeight = Math.min(window.innerHeight, visualHeight);
      const bottomClearance = window.innerWidth <= 1040 ? 12 : 0;
      const availableHeight = viewportHeight - stickyTop - bottomClearance;
      return stages.every((stage) => Math.ceil(stage.getBoundingClientRect().height) <= Math.floor(availableHeight) + 2);
    }

    function disablePipelineStack() {
      pipeline.classList.remove("is-scroll-stack");
      panels.forEach((panel) => panel.removeAttribute("inert"));
    }

    function ensurePipelineStillFits() {
      pipelineFitFrame = 0;
      if (isStacked() && !stackFitsViewport()) disablePipelineStack();
      requestPipelineUpdate();
      requestScrollUpdate();
    }

    function requestPipelineFitCheck() {
      if (!pipelineFitFrame) pipelineFitFrame = requestAnimationFrame(ensurePipelineStillFits);
    }

    function syncPipelineMode() {
      let canStack = CSS.supports("position", "sticky")
        && "inert" in HTMLElement.prototype
        && stackViewport.matches
        && !reducedMotion.matches;
      pipeline.classList.toggle("is-scroll-stack", canStack);

      if (canStack && !stackFitsViewport()) {
        canStack = false;
        disablePipelineStack();
      }

      if (!canStack) panels.forEach((panel) => panel.removeAttribute("inert"));
      requestPipelineUpdate();
      requestScrollUpdate();
    }

    function handlePipelineResize() {
      const nextWidth = window.innerWidth;
      if (Math.abs(nextWidth - viewportWidth) > 2) {
        viewportWidth = nextWidth;
        syncPipelineMode();
      } else {
        requestPipelineFitCheck();
      }
    }

    function restoreInitialPipelineFragment() {
      if (!window.location.hash) return;

      let fragmentId = "";
      try {
        fragmentId = decodeURIComponent(window.location.hash.slice(1));
      } catch {
        return;
      }

      const marker = markers.find((candidate) => candidate.id === fragmentId);
      if (!marker) return;

      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          const root = document.documentElement;
          root.classList.add("is-restoring-fragment");
          root.getBoundingClientRect();
          marker.scrollIntoView({ block: "start", behavior: "auto" });
          requestAnimationFrame(() => root.classList.remove("is-restoring-fragment"));
          requestPipelineUpdate();
          requestScrollUpdate();
        });
      });
    }

    links.forEach((link) => link.addEventListener("click", requestPipelineUpdate));
    window.addEventListener("scroll", requestPipelineUpdate, { passive: true });
    window.addEventListener("resize", handlePipelineResize, { passive: true });
    window.addEventListener("hashchange", requestPipelineUpdate);
    stackViewport.addEventListener("change", syncPipelineMode);
    reducedMotion.addEventListener("change", syncPipelineMode);
    window.visualViewport?.addEventListener("resize", requestPipelineFitCheck, { passive: true });

    if ("ResizeObserver" in window) {
      const resizeObserver = new ResizeObserver(requestPipelineFitCheck);
      resizeObserver.observe(pipeline);
    }

    syncPipelineMode();
    if (document.readyState === "complete") restoreInitialPipelineFragment();
    else window.addEventListener("load", restoreInitialPipelineFragment, { once: true });
  }

  function initCaseDrawer() {
    const scene = $("[data-case-drawer]");
    const runway = $("[data-case-drawer-runway]", scene || document);
    const surface = $(".case-drawer__surface", scene || document);
    const path = $("[data-case-drawer-path]", scene || document);
    const workflow = scene?.closest(".workflow");
    const forcedColors = window.matchMedia("(forced-colors: active)");
    const supported = CSS.supports("position", "sticky") && typeof SVGPathElement !== "undefined";

    if (!scene || !runway || !surface || !path || !workflow || !supported || reducedMotion.matches || forcedColors.matches) {
      return null;
    }

    let enabled = true;
    let lastProgress = -1;
    let surfaceHeight = window.innerHeight;
    let surfaceRect = null;

    function number(value) {
      return Math.max(0, Math.min(1, value)).toFixed(6).replace(/0+$/, "").replace(/\.$/, "");
    }

    function shapeAt(progressValue) {
      const shoulder = progressValue ** 4;
      const mouth = progressValue ** 8;
      return {
        top: 1 - progressValue,
        topLeft: 0.5 - 0.5 * mouth,
        topRight: 0.5 + 0.5 * mouth,
        leftControlA: 0.3 - 0.3 * shoulder,
        leftControlB: 0.4 - 0.4 * shoulder,
        rightControlA: 0.6 + 0.4 * shoulder,
        rightControlB: 0.7 + 0.3 * shoulder,
      };
    }

    function setPath(progressValue) {
      const shape = shapeAt(progressValue);

      path.setAttribute(
        "d",
        `M ${number(shape.topLeft)} ${number(shape.top)} C ${number(shape.leftControlA)} ${number(shape.top)} ${number(shape.leftControlB)} ${number(shape.top)} 0 1 L 1 1 C ${number(shape.rightControlA)} ${number(shape.top)} ${number(shape.rightControlB)} ${number(shape.top)} ${number(shape.topRight)} ${number(shape.top)} Z`,
      );
    }

    function update(force = false) {
      if (!enabled) return;

      const runwayRect = runway.getBoundingClientRect();
      surfaceRect = surface.getBoundingClientRect();
      surfaceHeight = surfaceRect.height || window.innerHeight;
      const distance = Math.max(1, runwayRect.height);
      const nextProgress = Math.max(0, Math.min(1, (surfaceHeight - runwayRect.top) / distance));

      if (!force && nextProgress === lastProgress) return;
      const exactEndpoint = nextProgress === 0 || nextProgress === 1;
      if (!force && !exactEndpoint && Math.abs(nextProgress - lastProgress) < 0.0005) return;
      lastProgress = nextProgress;
      setPath(nextProgress);
      scene.classList.toggle("is-drawing", nextProgress > 0 && nextProgress < 1);
    }

    function coversViewportPoint(x, y) {
      if (!enabled || lastProgress <= 0 || !surfaceRect) return false;
      if (x < surfaceRect.left || x > surfaceRect.right || y < surfaceRect.top || y > surfaceRect.bottom) return false;

      const normalizedX = (x - surfaceRect.left) / Math.max(1, surfaceRect.width);
      const normalizedY = (y - surfaceRect.top) / Math.max(1, surfaceRect.height);
      if (lastProgress === 1) return true;

      const shape = shapeAt(lastProgress);
      if (normalizedY < shape.top || normalizedY > 1) return false;

      const curveProgress = Math.cbrt((normalizedY - shape.top) / lastProgress);
      const inverse = 1 - curveProgress;
      const leftEdge = (inverse ** 3 * shape.topLeft)
        + (3 * inverse ** 2 * curveProgress * shape.leftControlA)
        + (3 * inverse * curveProgress ** 2 * shape.leftControlB);
      return normalizedX >= leftEdge && normalizedX <= 1 - leftEdge;
    }

    function ownsViewport() {
      return enabled
        && lastProgress > 0
        && surfaceRect
        && surfaceRect.bottom > 0
        && surfaceRect.top < surfaceHeight;
    }

    function isFullFrame() {
      return enabled && lastProgress === 1;
    }

    function disable() {
      if (!enabled) return;
      enabled = false;
      scene.hidden = true;
      scene.classList.remove("is-drawing");
      workflow.classList.remove("has-case-drawer");
      requestScrollUpdate();
    }

    function restoreInitialFragment() {
      if (!window.location.hash) return;

      let target = null;
      try {
        target = document.getElementById(decodeURIComponent(window.location.hash.slice(1)));
      } catch {
        return;
      }

      if (!target || !(scene.compareDocumentPosition(target) & Node.DOCUMENT_POSITION_FOLLOWING)) return;
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          const root = document.documentElement;
          root.classList.add("is-restoring-fragment");
          root.getBoundingClientRect();
          target.scrollIntoView({ block: "start", behavior: "auto" });
          requestAnimationFrame(() => root.classList.remove("is-restoring-fragment"));
          requestScrollUpdate();
        });
      });
    }

    scene.hidden = false;
    workflow.classList.add("has-case-drawer");
    setPath(0);
    update(true);
    if (document.readyState === "complete") restoreInitialFragment();
    else window.addEventListener("load", restoreInitialFragment, { once: true });

    reducedMotion.addEventListener("change", (event) => {
      if (event.matches) disable();
    });
    forcedColors.addEventListener("change", (event) => {
      if (event.matches) disable();
    });

    return { update, coversViewportPoint, ownsViewport, isFullFrame };
  }

  let scrollFrame = 0;
  function updateScrollState() {
    scrollFrame = 0;
    const root = document.documentElement;
    const maxScroll = root.scrollHeight - root.clientHeight;
    const percent = maxScroll > 0 ? Math.round((window.scrollY / maxScroll) * 100) : 0;

    header?.classList.toggle("is-scrolled", window.scrollY > 16);
    if (progress) progress.textContent = `${Math.max(0, Math.min(100, percent))}%`;
    if (hudPosition) hudPosition.textContent = `Y: ${String(Math.round(window.scrollY)).padStart(4, "0")}`;

    caseDrawerController?.update();

    const drawerOwnsViewport = caseDrawerController?.ownsViewport();
    const drawerIsFullFrame = caseDrawerController?.isFullFrame();
    const drawerCoversHeader = caseDrawerController?.coversViewportPoint(window.innerWidth / 2, 86);
    const themeSection = elementAtViewportPoint("[data-section-theme]", window.innerWidth / 2, 86);
    setTheme(
      drawerOwnsViewport
        ? (drawerIsFullFrame || drawerCoversHeader ? "dark" : "light")
        : themeSection?.dataset.sectionTheme || "dark",
    );

    const labelX = Math.min(60, window.innerWidth / 2);
    const labelY = window.innerHeight / 2;
    const drawerCoversLabel = caseDrawerController?.coversViewportPoint(labelX, labelY);
    const labelSection = elementAtViewportPoint("[data-section-label]", labelX, labelY);
    if (hudSection && (drawerOwnsViewport || labelSection?.dataset.sectionLabel)) {
      hudSection.textContent = drawerOwnsViewport
        ? (drawerIsFullFrame || drawerCoversLabel ? "§03 · CASE STUDY" : "§02 · PIPELINE")
        : labelSection.dataset.sectionLabel;
    }

  }

  function requestScrollUpdate() {
    if (!scrollFrame) scrollFrame = requestAnimationFrame(updateScrollState);
  }

  function initScrollState() {
    updateScrollState();
    window.addEventListener("scroll", requestScrollUpdate, { passive: true });
    window.addEventListener("resize", requestScrollUpdate, { passive: true });
    window.addEventListener("pageshow", requestScrollUpdate);
    window.visualViewport?.addEventListener("resize", requestScrollUpdate, { passive: true });
  }

  function initReveals() {
    const elements = $$(".reveal");
    if (!elements.length) return;

    if (reducedMotion.matches || !("IntersectionObserver" in window)) {
      elements.forEach((element) => element.classList.add("is-visible"));
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          entry.target.classList.add("is-visible");
          observer.unobserve(entry.target);
        });
      },
      { rootMargin: "0px 0px -9%", threshold: 0.08 },
    );

    elements.forEach((element) => observer.observe(element));
  }

  function initCardGlow() {
    if (!finePointer.matches || reducedMotion.matches) return;

    $$(".principle-card").forEach((card) => {
      card.addEventListener("pointermove", (event) => {
        const rect = card.getBoundingClientRect();
        card.style.setProperty("--pointer-x", `${event.clientX - rect.left}px`);
        card.style.setProperty("--pointer-y", `${event.clientY - rect.top}px`);
      });
    });
  }

  function initDemoParallax() {
    if (!finePointer.matches || reducedMotion.matches) return;

    const hero = $("#hero");
    const windows = $$('[data-depth]');
    if (!hero || !windows.length) return;

    hero.addEventListener("pointermove", (event) => {
      const rect = hero.getBoundingClientRect();
      const x = (event.clientX - rect.left) / rect.width - 0.5;
      const y = (event.clientY - rect.top) / rect.height - 0.5;

      windows.forEach((windowElement) => {
        const depth = Number(windowElement.dataset.depth) || 1;
        const translateX = x * 7 * depth;
        const translateY = y * 7 * depth;
        windowElement.style.transform = `translate3d(${translateX.toFixed(2)}px, ${translateY.toFixed(2)}px, 0)`;
      });
    });

    hero.addEventListener("pointerleave", () => {
      windows.forEach((windowElement) => {
        windowElement.style.transform = "";
      });
    });
  }

  function initTilt() {
    if (!finePointer.matches || reducedMotion.matches) return;

    $$('[data-tilt]').forEach((element) => {
      element.addEventListener("pointermove", (event) => {
        const rect = element.getBoundingClientRect();
        const x = (event.clientX - rect.left) / rect.width - 0.5;
        const y = (event.clientY - rect.top) / rect.height - 0.5;
        element.style.setProperty("--tilt-x", `${(-y * 6).toFixed(2)}deg`);
        element.style.setProperty("--tilt-y", `${(x * 6).toFixed(2)}deg`);
      });

      element.addEventListener("pointerleave", () => {
        element.style.removeProperty("--tilt-x");
        element.style.removeProperty("--tilt-y");
      });
    });
  }

  function legacyCopy(element) {
    const selection = window.getSelection();
    if (!selection) return false;

    const range = document.createRange();
    range.selectNodeContents(element);
    selection.removeAllRanges();
    selection.addRange(range);

    let copied = false;
    try {
      copied = document.execCommand("copy");
    } catch {
      copied = false;
    } finally {
      selection.removeAllRanges();
    }
    return copied;
  }

  function initCopyButton() {
    const button = $("[data-copy-command]");
    const source = $("#install-command");
    const label = $("[data-copy-label]", button || document);
    if (!button || !source || !label) return;

    let resetTimer = 0;
    button.addEventListener("click", async () => {
      window.clearTimeout(resetTimer);
      button.classList.remove("is-copied", "is-error");

      let copied = false;
      try {
        if (navigator.clipboard?.writeText && window.isSecureContext) {
          await navigator.clipboard.writeText(source.textContent.trim());
          copied = true;
        } else {
          copied = legacyCopy(source);
        }
      } catch {
        copied = false;
      }

      button.classList.add(copied ? "is-copied" : "is-error");
      label.textContent = copied ? "Copied" : "Copy failed";
      resetTimer = window.setTimeout(() => {
        button.classList.remove("is-copied", "is-error");
        label.textContent = "Copy";
      }, 2000);
    });
  }

  function initCanvas() {
    const canvas = $("#pcb-canvas");
    const hero = $("#hero");
    if (!canvas || !hero) return null;

    const context = canvas.getContext("2d", { alpha: true });
    if (!context) return null;

    let width = 0;
    let height = 0;
    let dpr = 1;
    let traces = [];
    let backbone = null;
    let theme = document.body.dataset.theme || "dark";
    let resizeTimer = 0;
    let seed = 0x0c0d1;

    const directions = [
      [1, 0], [Math.SQRT1_2, Math.SQRT1_2], [0, 1], [-Math.SQRT1_2, Math.SQRT1_2],
      [-1, 0], [-Math.SQRT1_2, -Math.SQRT1_2], [0, -1], [Math.SQRT1_2, -Math.SQRT1_2],
    ];

    function random() {
      seed = (seed * 1664525 + 1013904223) >>> 0;
      return seed / 4294967296;
    }

    function makeTrace(long = false) {
      const grid = 28;
      const edge = random() > 0.5;
      let x = edge ? random() * width : (random() > 0.5 ? random() * width * 0.18 : width * (0.82 + random() * 0.18));
      let y = edge ? (random() > 0.5 ? random() * height * 0.2 : height * (0.8 + random() * 0.2)) : random() * height;
      x = Math.round(x / grid) * grid;
      y = Math.round(y / grid) * grid;

      const points = [{ x, y }];
      let direction = Math.floor(random() * directions.length);
      const segments = long ? 9 : 3 + Math.floor(random() * 4);

      for (let index = 0; index < segments; index += 1) {
        const turn = random();
        if (turn > 0.42 && turn < 0.72) direction = (direction + (random() > 0.5 ? 1 : 7)) % 8;
        else if (turn >= 0.72) direction = (direction + (random() > 0.5 ? 2 : 6)) % 8;

        const length = (2 + Math.floor(random() * (long ? 6 : 4))) * grid;
        x = Math.max(-grid, Math.min(width + grid, x + directions[direction][0] * length));
        y = Math.max(-grid, Math.min(height + grid, y + directions[direction][1] * length));
        points.push({ x, y });
      }

      return { points, tone: Math.floor(random() * 3) };
    }

    function rebuild() {
      seed = 0x0c0d1;
      traces = [];
      const count = Math.max(8, Math.min(18, Math.round((width * height) / 125000)));
      for (let index = 0; index < count; index += 1) traces.push(makeTrace());
      backbone = makeTrace(true);
    }

    function resize() {
      width = window.innerWidth;
      height = window.innerHeight;
      const requestedDpr = Math.min(window.devicePixelRatio || 1, 1.5);
      const pixelBudgetDpr = Math.sqrt(4500000 / Math.max(1, width * height));
      dpr = Math.min(requestedDpr, pixelBudgetDpr);
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
      context.setTransform(dpr, 0, 0, dpr, 0, 0);
      rebuild();
      render(performance.now());
    }

    function palette() {
      return theme === "light"
        ? ["#08785f", "#0b7370", "#5b4fc8"]
        : ["#62b5ff", "#5de0d0", "#a99aff"];
    }

    function drawTrace(trace, alpha = 0.13) {
      const color = palette()[trace.tone] || palette()[0];
      context.beginPath();
      trace.points.forEach((point, index) => {
        if (index === 0) context.moveTo(point.x, point.y);
        else context.lineTo(point.x, point.y);
      });
      context.lineWidth = 1;
      context.lineCap = "round";
      context.lineJoin = "round";
      context.globalAlpha = alpha;
      context.strokeStyle = color;
      context.stroke();

      for (const point of [trace.points[0], trace.points.at(-1)]) {
        context.beginPath();
        context.arc(point.x, point.y, 3.8, 0, Math.PI * 2);
        context.globalAlpha = alpha * 1.7;
        context.strokeStyle = color;
        context.stroke();
      }
    }

    function pointOnTrace(trace, amount) {
      const segments = [];
      let total = 0;
      for (let index = 1; index < trace.points.length; index += 1) {
        const start = trace.points[index - 1];
        const end = trace.points[index];
        const length = Math.hypot(end.x - start.x, end.y - start.y);
        segments.push({ start, end, length });
        total += length;
      }

      let target = total * amount;
      for (const segment of segments) {
        if (target <= segment.length) {
          const ratio = segment.length ? target / segment.length : 0;
          return {
            x: segment.start.x + (segment.end.x - segment.start.x) * ratio,
            y: segment.start.y + (segment.end.y - segment.start.y) * ratio,
          };
        }
        target -= segment.length;
      }
      return trace.points.at(-1);
    }

    function render(timestamp) {
      context.clearRect(0, 0, width, height);
      traces.forEach((trace) => drawTrace(trace));
      if (backbone) {
        drawTrace(backbone, 0.18);
        if (!reducedMotion.matches && finePointer.matches && width >= 768) {
          const point = pointOnTrace(backbone, (timestamp * 0.00008) % 1);
          const color = palette()[backbone.tone] || palette()[0];
          const glow = context.createRadialGradient(point.x, point.y, 0, point.x, point.y, 10);
          glow.addColorStop(0, color);
          glow.addColorStop(1, "transparent");
          context.globalAlpha = 0.8;
          context.fillStyle = glow;
          context.beginPath();
          context.arc(point.x, point.y, 10, 0, Math.PI * 2);
          context.fill();
        }
      }
      context.globalAlpha = 1;
    }

    window.addEventListener("resize", () => {
      window.clearTimeout(resizeTimer);
      resizeTimer = window.setTimeout(resize, 160);
    }, { passive: true });

    reducedMotion.addEventListener("change", () => {
      render(performance.now());
    });

    finePointer.addEventListener("change", () => {
      render(performance.now());
    });

    resize();

    return {
      setTheme(nextTheme) {
        theme = nextTheme;
        render(performance.now());
      },
    };
  }

  initNavigation();
  initPipelineStack();
  initReveals();
  initCardGlow();
  initDemoParallax();
  initTilt();
  initCopyButton();
  canvasController = initCanvas();
  caseDrawerController = initCaseDrawer();
  initScrollState();
})();

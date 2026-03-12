(function () {
  const root = document.documentElement;
  const storageKey = "cortex-theme-mode";
  const themeOptions = Array.from(document.querySelectorAll(".theme-option"));

  function applyTheme(mode) {
    if (mode === "system") {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      root.setAttribute("data-theme", prefersDark ? "dark" : "light");
    } else {
      root.setAttribute("data-theme", mode);
    }

    localStorage.setItem(storageKey, mode);
    themeOptions.forEach((opt) => opt.classList.toggle("is-active", opt.dataset.themeMode === mode));
  }

  if (themeOptions.length > 0) {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    if (typeof media.addEventListener === "function") {
      media.addEventListener("change", () => {
        if (localStorage.getItem(storageKey) === "system") applyTheme("system");
      });
    }

    themeOptions.forEach((opt) => opt.addEventListener("click", () => applyTheme(opt.dataset.themeMode)));
    applyTheme(localStorage.getItem(storageKey) || "dark");
  }

  function makeCopyButton(getText) {
    const btn = document.createElement("button");
    btn.className = "copy-btn";
    btn.type = "button";
    btn.setAttribute("aria-label", "Copy command");

    const copyIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>`;
    const doneIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
    btn.innerHTML = copyIcon;

    btn.addEventListener("click", async () => {
      const text = (getText() || "").trim();
      if (!text) return;

      try {
        await navigator.clipboard.writeText(text);
        btn.classList.add("copied");
        btn.innerHTML = doneIcon;
        window.setTimeout(() => {
          btn.classList.remove("copied");
          btn.innerHTML = copyIcon;
        }, 1600);
      } catch (_) {}
    });

    return btn;
  }

  document.querySelectorAll("pre").forEach((pre) => {
    if (pre.querySelector(".copy-btn")) return;
    const code = pre.querySelector("code");
    if (!code) return;
    pre.appendChild(makeCopyButton(() => code.innerText));
  });

  document.querySelectorAll(".command-card").forEach((card) => {
    if (card.querySelector(".copy-btn")) return;
    const code = card.querySelector("code");
    if (!code) return;
    card.appendChild(makeCopyButton(() => code.innerText));
  });

  function setIndicator(nav, indicator, link) {
    if (!nav || !indicator || !link) return;
    const target = link.querySelector(".nav-text") || link;
    const navRect = nav.getBoundingClientRect();
    const textRect = target.getBoundingClientRect();
    const indicatorRect = indicator.getBoundingClientRect();
    const dotSize = indicatorRect.height || 4;
    const y = ((textRect.top + textRect.bottom) / 2) - navRect.top - dotSize / 2;
    indicator.style.opacity = "1";
    indicator.style.transform = `translateY(${Math.max(0, y)}px)`;
  }

  document.querySelectorAll(".rail-nav").forEach((nav) => {
    const links = Array.from(nav.querySelectorAll("a[href^='#']"));
    const indicator = nav.querySelector(".nav-indicator");
    if (links.length === 0) return;

    const mode = nav.dataset.mode || "anchors";

    function activateLink(link) {
      links.forEach((item) => item.classList.toggle("is-active", item === link));
      setIndicator(nav, indicator, link);
      if (nav.scrollHeight > nav.clientHeight) {
        link.scrollIntoView({ block: "nearest", behavior: "auto" });
      }
    }

    if (mode === "sections") {
      const sections = Array.from(document.querySelectorAll(".panel-section"));
      if (sections.length === 0) return;

      function activateSection(id, replaceHash = true, resetScroll = true) {
        const target = sections.find((section) => section.id === id) || sections[0];
        sections.forEach((section) => {
          const active = section === target;
          section.classList.toggle("is-active", active);
          section.style.display = active ? "block" : "none";
        });

        const activeLink = links.find((link) => link.getAttribute("href") === `#${target.id}`) || links[0];
        activateLink(activeLink);

        if (replaceHash) {
          history.replaceState(null, "", `#${target.id}`);
        }

        if (resetScroll) {
          window.scrollTo({ top: 0, behavior: "auto" });
        }
      }

      links.forEach((link) => {
        link.addEventListener("click", (event) => {
          event.preventDefault();
          activateSection(link.getAttribute("href").slice(1), false, true);
        });
      });

      const syncFromHash = (resetScroll = false) => {
        const hash = (window.location.hash || "").replace(/^#/, "");
        const target = hash && document.getElementById(hash) ? hash : sections[0].id;
        activateSection(target, false, resetScroll);
      };

      window.addEventListener("hashchange", () => syncFromHash(true));
      window.addEventListener("resize", () => {
        const active = links.find((link) => link.classList.contains("is-active")) || links[0];
        activateLink(active);
      });

      syncFromHash(false);
      return;
    }

    function syncAnchors() {
      const hash = window.location.hash;
      const active = links.find((link) => link.getAttribute("href") === hash) || links[0];
      activateLink(active);
    }

    links.forEach((link) => {
      link.addEventListener("click", () => {
        window.setTimeout(syncAnchors, 20);
      });
    });

    window.addEventListener("hashchange", syncAnchors);
    window.addEventListener("resize", () => {
      const active = links.find((link) => link.classList.contains("is-active")) || links[0];
      activateLink(active);
    });

    syncAnchors();
  });
})();

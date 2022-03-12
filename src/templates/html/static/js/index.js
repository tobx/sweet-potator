(() => {
  const selectors = {
    favorites: "body > header > nav > .favorites",
    recipe: ".recipes > .list > ul > li",
    recipeCount: "main > .recipes > .list > .count",
    tag: ".tags ul li .tag",
    tagReset: ".tags ul li .reset",
  };

  class Tag {
    constructor() {
      this.isActive = false;
      this.elements = [];
    }

    refresh() {
      for (const element of this.elements) {
        element.classList.toggle("active", this.isActive);
      }
    }

    toggle(force) {
      this.isActive = force === undefined ? !this.isActive : force;
    }
  }

  class TaggedElement {
    constructor(element) {
      this.element = element;
      this.tags = queryElementTags(element);
    }
  }

  class TagManager {
    constructor() {
      this.tags = new Map();
      this.tagged = [];
    }

    addTagElement(name, element) {
      if (!this.tags.has(name)) {
        this.tags.set(name, new Tag());
      }
      this.tags.get(name).elements.push(element);
    }

    addTaggedElement(element) {
      this.tagged.push(new TaggedElement(element));
    }

    addEventListeners() {
      window.addEventListener("hashchange", () => this.updateFromUrlHash());
      for (const tag of this.tags.values()) {
        for (const element of tag.elements) {
          element.addEventListener("click", () => {
            tag.toggle();
            this.updateUrlHash();
          });
        }
      }
      for (const element of document.querySelectorAll(selectors.tagReset)) {
        element.addEventListener("click", () => this.reset());
      }
    }

    refresh() {
      for (const tag of this.tags.values()) {
        tag.refresh();
      }
      let count = 0;
      for (const tagged of this.tagged) {
        const show = Array.from(this.tags)
          .filter(([, tag]) => tag.isActive)
          .every(([name]) => tagged.tags.has(name));
        tagged.element.classList.toggle("hidden", !show);
        if (show) {
          count++;
        }
      }
      const recipeCount = document.querySelector(selectors.recipeCount);
      recipeCount.querySelector(".value").textContent = count;
      const word = recipeCount.querySelector(".recipes");
      word.textContent = word.dataset[count === 1 ? "singular" : "plural"];
    }

    reset() {
      for (const tag of this.tags.values()) {
        tag.toggle(false);
      }
      this.updateUrlHash();
    }

    updateUrlHash() {
      const params = new URLSearchParams(location.hash.slice(1));
      if ([...this.tags.values()].some((tag) => tag.isActive)) {
        params.set(
          "tags",
          this.constructor.encodeTagNames(
            [...this.tags]
              .filter(([, tag]) => tag.isActive)
              .map(([name]) => name)
          )
        );
      } else {
        params.delete("tags");
      }
      // not using `params.toString()` to have pretty urls (browser already takes care of url encoding)
      const hash = [...params].map((entry) => entry.join("=")).join("&");
      let path = location.pathname;
      const isRecipePage = document.querySelector("main > .recipe") !== null;
      if (isRecipePage) {
        let index = path.lastIndexOf("/");
        index = path.lastIndexOf("/", index - 1);
        path = path.slice(0, index + 1);
      }
      if (hash !== "") {
        path += "#" + hash;
      }
      history.pushState("", "", path);
      if (isRecipePage) {
        location.reload();
      } else {
        this.updateFromUrlHash();
      }
    }

    updateFromUrlHash() {
      const params = new URLSearchParams(location.hash.slice(1));
      const tagNames = this.constructor.decodeTagNames(
        params.get("tags") ?? ""
      );
      for (const [name, tag] of this.tags) {
        const activate = tagNames.has(name);
        if (activate !== tag.isActive) {
          tag.toggle(activate);
        }
      }
      this.refresh();
    }

    static encodeTagNames(tags) {
      return tags
        .map((text) => text.replace("\\", "\\\\").replace(",", "\\,"))
        .join(",");
    }

    static decodeTagNames(text) {
      const tags = new Set();
      let escape = false;
      let current = "";
      for (const c of text) {
        if (escape) {
          current += c;
          escape = false;
        } else if (c === "\\") {
          escape = true;
        } else if (c === ",") {
          tags.add(current);
          current = "";
        } else {
          current += c;
        }
      }
      if (current !== "") {
        tags.add(current);
      }
      return tags;
    }
  }

  function queryElementTags(root) {
    const tags = new Set();
    for (const element of root.querySelectorAll(selectors.tag)) {
      tags.add(element.textContent);
    }
    return tags;
  }

  function queryTagElements(root) {
    let elements = root.querySelectorAll(selectors.tag);
    return Array.prototype.map.call(elements, (e) => [e.textContent, e]);
  }

  function initialize() {
    const tags = new TagManager();
    for (const [name, element] of queryTagElements(document)) {
      tags.addTagElement(name, element);
    }
    const favoritesElement = document.querySelector(selectors.favorites);
    tags.addTagElement(favoritesElement.dataset.tagName, favoritesElement);
    const recipes = document.querySelectorAll(selectors.recipe);
    for (const element of recipes) {
      tags.addTaggedElement(element);
    }
    tags.addEventListeners();
    tags.updateFromUrlHash();
  }

  window.addEventListener("DOMContentLoaded", initialize);
})();

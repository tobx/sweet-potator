(() => {
  const selectors = {
    favorites: "body > header > nav > .favorites",
    recipe: ".recipes > .list > ul > li",
    tag: ".tags ul li .tag",
    tagReset: ".tags ul li .reset",
  };

  class Tag {
    constructor() {
      this.isActive = false;
      this.elements = [];
    }

    toggle(force) {
      this.isActive = force === undefined ? !this.isActive : force;
      for (const element of this.elements) {
        element.classList.toggle("active", this.isActive);
      }
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
      for (const tag of this.tags.values()) {
        for (const element of tag.elements) {
          element.addEventListener("click", () => {
            tag.toggle();
            this.refresh();
          });
        }
      }
      for (const element of document.querySelectorAll(selectors.tagReset)) {
        element.addEventListener("click", () => {
          this.reset();
        });
      }
    }

    refresh() {
      for (const tagged of this.tagged) {
        const show = Array.from(this.tags)
          .filter(([, tag]) => tag.isActive)
          .every(([name]) => tagged.tags.has(name));
        tagged.element.classList.toggle("hidden", !show);
      }
    }

    reset() {
      for (const tag of this.tags.values()) {
        tag.toggle(false);
      }
      this.refresh();
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
    for (const element of document.querySelectorAll(selectors.favorites)) {
      tags.addTagElement("favorite", element);
    }
    const recipes = document.querySelectorAll(selectors.recipe);
    for (const element of recipes) {
      tags.addTaggedElement(element);
    }
    tags.addEventListeners();
  }

  window.addEventListener("DOMContentLoaded", initialize);
})();

(() => {
  const selectors = {
    favorites: "body > header > nav > .favorites",
    ingredientQuantity: "main > .recipe > .ingredients .quantity > .value",
    random: "main > .recipes > .list .random",
    recipe: "main > .recipes > .list > ul > li",
    recipeCount: "main > .recipes > .list > .count",
    tag: ".tags ul li .tag",
    tagReset: ".tags ul li .reset",
    yield: "main .recipe > .metadata > .yield > .content > .value",
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

    random() {
      const activateTagNames = [...this.tags]
        .filter(([name, tag]) => tag.isActive)
        .map(([name]) => name);
      const elements = this.tagged
        .filter((tagEl) =>
          activateTagNames.every((name) => tagEl.tags.has(name))
        )
        .map((tagEl) => tagEl.element);
      return elements[Math.floor(Math.random() * elements.length)];
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
      if (recipeCount !== null) {
        recipeCount.querySelector(".value").textContent = count;
        const word = recipeCount.querySelector(".recipes");
        word.textContent = word.dataset[count === 1 ? "singular" : "plural"];
      }
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
      if (isRecipePage()) {
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

  // greatest common divisor
  function gcd(a, b) {
    while (b !== 0) {
      t = b;
      b = a % b;
      a = t;
    }
    return a;
  }

  class Fraction {
    constructor(numer, denom) {
      this.numer = numer;
      this.denom = denom;
    }

    reduce() {
      const divisor = gcd(this.numer, this.denom);
      return new Fraction(this.numer / divisor, this.denom / divisor);
    }

    multiply(fraction) {
      return new Fraction(
        this.numer * fraction.numer,
        this.denom * fraction.denom
      );
    }

    scaleNumber(number) {
      return (number * this.numer) / this.denom;
    }

    toString() {
      let value = [];
      let numer = this.numer;
      let denom = this.denom;
      if (numer >= denom) {
        const remainder = numer % denom;
        value.push((numer - remainder) / denom);
        numer = remainder;
      }
      if (numer > 0) {
        value.push(numer + "\u2044" + this.denom);
      }
      return value.join(" ");
    }

    static parseFrom(text) {
      const [numer, denom] = text.split("\u2044");
      return denom === undefined
        ? null
        : new this(parseInt(numer), parseInt(denom));
    }
  }

  class Quantity {
    constructor(element, decimalSeparator) {
      this.element = element;
      this.decimalSeparator = decimalSeparator;
      const value = element.textContent;
      const fraction = Fraction.parseFrom(value, decimalSeparator);
      this.defaultValue =
        fraction ?? parseFloat(value.replace(decimalSeparator, "."));
      this.currentValue = this.defaultValue;
    }

    isFraction() {
      return this.defaultValue instanceof Fraction;
    }

    reset() {
      this.currentValue = this.defaultValue;
      this.refresh();
    }

    refresh() {
      if (this.isFraction()) {
        this.element.textContent = this.currentValue.toString();
      } else {
        this.element.textContent = String(
          Math.round(this.currentValue * 100) / 100
        ).replace(".", this.decimalSeparator);
      }
    }

    scale(fraction) {
      if (this.isFraction()) {
        this.currentValue = this.defaultValue.multiply(fraction).reduce();
      } else {
        this.currentValue = fraction.scaleNumber(this.defaultValue);
      }
      this.refresh();
    }
  }

  class IngredientManager {
    constructor(config) {
      const yields = document.querySelector(selectors.yield);
      yields
        .querySelector(".decrease")
        .addEventListener("click", () => this.decrease());
      yields
        .querySelector(".increase")
        .addEventListener("click", () => this.increase());
      this.yieldDigits = yields.querySelector(".digits");
      this.yieldDigits.addEventListener("click", () => this.reset());
      this.defaultYield = parseInt(this.yieldDigits.textContent);
      this.currentYield = this.defaultYield;
      this.quantities = [
        ...document.querySelectorAll(selectors.ingredientQuantity),
      ]
        .filter((element) => element.textContent !== null)
        .map((element) => new Quantity(element, config.decimalSeparator));
    }

    decrease() {
      if (this.currentYield > 1) {
        this.currentYield--;
        this.refresh();
      }
    }

    increase() {
      this.currentYield++;
      this.refresh();
    }

    refresh() {
      const yieldIsDefault = this.currentYield === this.defaultYield;
      this.yieldDigits.textContent = this.currentYield;
      this.yieldDigits.classList.toggle("default", yieldIsDefault);
      if (yieldIsDefault) {
        for (const quantity of this.quantities) {
          quantity.reset();
        }
      } else {
        const fraction = new Fraction(this.currentYield, this.defaultYield);
        for (const quantity of this.quantities) {
          quantity.scale(fraction);
        }
      }
    }

    reset() {
      this.currentYield = this.defaultYield;
      this.refresh();
    }
  }

  function addCollapseEventHandlers() {
    for (const trigger of document.querySelectorAll(".collapse-trigger")) {
      trigger.addEventListener("click", () => {
        const isCollapsed = trigger.classList.toggle("collapsed");
        trigger.textContent = isCollapsed ? "+" : "−";
        document
          .querySelector(trigger.dataset.collapseSelector)
          .classList.toggle("collapsed", isCollapsed);
      });
    }
  }

  function isRecipePage() {
    return document.querySelector("main > .recipe") !== null;
  }

  function isRecipesPage() {
    return document.querySelector("main > .recipes") !== null;
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
    const config = Object.fromEntries(
      Object.entries(document.getElementById("config").dataset)
    );
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
    if (isRecipesPage()) {
      document.querySelector(selectors.random).addEventListener("click", () => {
        const href = tags.random().querySelector("a").getAttribute("href");
        location.assign(location.pathname + href);
      });
    }
    if (isRecipePage()) {
      const ingredients = new IngredientManager(config);
      ingredients.reset();
    }
    addCollapseEventHandlers();
  }

  window.addEventListener("DOMContentLoaded", initialize);
})();

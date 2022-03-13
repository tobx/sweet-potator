# {{ title }}

## Recipes

{% for recipe in recipes -%}
  - [{{ recipe.title }}]({{ recipe.path | escape_xml | safe }})
{% endfor %}

This is just a proof of concept for [Sweet Potator](https://github.com/tobx/sweet-potator).

For a more advanced example look into the official templates in `src/templates`.

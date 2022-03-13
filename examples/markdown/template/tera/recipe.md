# {{ recipe.title }}

{% if image_path is string -%}
  ![{{ recipe.title }}](../{{ image_path }})

{% endif -%}

This is just a proof of concept for [Sweet Potator](https://github.com/tobx/sweet-potator).

For a more advanced example look into the official templates in `src/templates`.

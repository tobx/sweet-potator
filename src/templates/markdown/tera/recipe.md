{% import "macros/list.md" as list %}
{% import "macros/table.md" as table %}

{% extends "blocks/base.md" %}

{% block main -%}
# {{ recipe.title }}

{% if image_path is string -%}
  ![{{ recipe.title }}](../{{ image_path }}){{ lf ~ lf }}
{%- endif -%}

Servings: {{ recipe.metadata.servings }}

## Ingredients
{{ table::table(list = recipe.ingredients) }}
## Instructions
{{ list::list(list = recipe.instructions) }}
{%- if recipe.metadata.source is object -%}
  {{ lf }}
  {%- if recipe.metadata.source.link is defined -%}
    Source: [{{ recipe.metadata.source.link.name }}]({{ recipe.metadata.source.link.url | urlencode | safe }})
  {%- else -%}
    Author: {{ recipe.metadata.source.author }}
  {%- endif %}
{% endif -%}
{%- endblock main %}

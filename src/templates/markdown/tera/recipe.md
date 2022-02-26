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
{%- if recipe.notes | length > 0 -%}
  {{ lf }}## Notes
  {%- for item in recipe.notes %}
    {%- if loop.first %}{{ lf }}{% endif -%}
    {{ lf }}- {{ item ~ lf }}
  {%- endfor %}
{%- endif %}
## Instructions
{{ list::list(list = recipe.instructions) }}
{%- set source = recipe.metadata.source %}
{%- if source is object %}
  {{ lf }}
  {%- if source.author is defined -%}
    Author: {{ source.author }}
  {%- elif source.book is defined -%}
    Source: {{ source.book }}
  {%- else -%}
    Source: [{{ source.link.name }}]({{ source.link.url | urlencode | safe }})
  {%- endif %}
{% endif -%}
{%- endblock main %}

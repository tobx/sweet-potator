{% import "macros/list.md" as list %}
{% import "macros/table.md" as table %}

{% extends "blocks/base.md" %}

{% block main -%}
# {{ recipe.title }}

{% if image_path is string -%}
  ![{{ recipe.title }}](../{{ image_path | urlencode | safe }}){{ lf ~ lf }}
{%- endif -%}

{%- set servings = recipe.metadata.servings -%}
Servings: {{ servings.value }}
{%- if servings.unit is string -%}
  {{ " " }}{{ servings.unit }}
{%- endif %}

{%- set duration = recipe.metadata.duration %}
{%- if duration is object -%}
  {{ "  " ~ lf }}Preparation:
  {%- if duration.hours > 0 -%}
    {{ " " ~ duration.hours }} hr
    {%- if duration.hours > 1 %}s{% endif %}
  {%- endif %}
  {%- if duration.minutes > 0 -%}
    {{ " " ~ duration.minutes }} min
    {%- if duration.minutes > 1 %}s{% endif %}
  {%- endif %}
{%- endif %}

## Ingredients
{{ table::table(list = recipe.ingredients) }}
{%- if recipe.notes | length > 0 -%}
  {{ lf }}## Notes
  {%- for item in recipe.notes %}
    {%- if loop.first %}{{ lf }}{% endif -%}
    {{ lf }}- {{ item ~ lf }}
  {%- endfor %}
{%- endif -%}
{{ lf -}}

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

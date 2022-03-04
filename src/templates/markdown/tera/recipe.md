{% import "macros/list.md" as list %}
{% import "macros/table.md" as table %}

{% extends "blocks/base.md" %}

{% block main -%}
# {{ recipe.title }}

{% if image_path is string -%}
  ![{{ recipe.title }}](../{{ image_path | escape_xml | safe }}){{ lf ~ lf }}
{%- endif -%}

{%- set yield = recipe.metadata.yield -%}
Servings: {{ yield.value }}
{%- if yield.unit is string -%}
  {{ " " }}{{ yield.unit }}
{%- endif %}

{%- set duration = recipe.metadata.duration %}
{%- if duration is object -%}
  {{ "  " ~ lf }}Preparation:
  {%- if duration.hours > 0 -%}
    {{ " " ~ duration.hours }} hr{{ duration.hours | pluralize }}
  {%- endif %}
  {%- if duration.minutes > 0 -%}
    {{ " " ~ duration.minutes }} min{{ duration.minutes | pluralize }}
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
    Source: [{{ source.link.name }}]({{ source.link.url | escape_xml | safe }})
  {%- endif %}
{% endif -%}
{%- endblock main %}

{% import "macros/list.md" as list %}
{% import "macros/table.md" as table %}

{% extends "blocks/base.md" %}

{% block main %}
{%- set lang = lang.recipe %}
# {{ recipe.title }}

{% if image_path is string -%}
  ![{{ recipe.title }}](../{{ image_path | escape_xml | safe }}){{ lf ~ lf }}
{%- endif -%}

{%- set yield = recipe.metadata.yield -%}
  {{ lang.metadata_servings }}: {{ yield.value }}
{%- if yield.unit is string -%}
  {{ " " }}{{ yield.unit }}
{%- endif %}

{%- set duration = recipe.metadata.duration %}
{%- if duration is object -%}
  {{ "  " ~ lf ~ lang.metadata_preparation_time }}:
  {%- if duration.hours > 0 -%}
    {{ " " ~ duration.hours }} {{
      duration.hours | pluralize(
        singular = lang.metadata_hour,
        plural = lang.metadata_hours
      )
    }}
  {%- endif %}
  {%- if duration.minutes > 0 -%}
    {{ " " ~ duration.minutes }} {{ 
      duration.minutes | pluralize(
        singular = lang.metadata_minute,
        plural = lang.metadata_minutes
      )
    }}
  {%- endif %}
{%- endif %}

## {{ lang.heading_ingredients }}
{{ table::table(list = recipe.ingredients) }}
{%- if recipe.notes | length > 0 -%}
  {{ lf }}## {{ lang.heading_notes }}
  {%- for item in recipe.notes %}
    {%- if loop.first %}{{ lf }}{% endif -%}
    {{ lf }}- {{ item ~ lf }}
  {%- endfor %}
{%- endif -%}
{{ lf -}}

## {{ lang.heading_instructions }}
{{ list::list(list = recipe.instructions) }}
{%- set source = recipe.metadata.source %}
{%- if source is object %}
  {{ lf }}
  {%- if source.author is defined -%}
    {{ lang.metadata_author }}: {{ source.author }}
  {%- elif source.book is defined -%}
    {{ lang.metadata_source }}: {{ source.book }}
  {%- else -%}
    {{ lang.metadata_source }}: [{{ source.link.name }}]({{ source.link.url | escape_xml | safe }})
  {%- endif %}
{% endif -%}
{%- endblock main %}

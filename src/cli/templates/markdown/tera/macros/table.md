{% macro quantity(quantity) -%}
  {{ quantity.value }}
  {%- if quantity.unit is string -%}
    {{ " " ~ quantity.unit }}
  {%- endif %}
  {%- if quantity.note is string -%}
    {{ " " }}({{ quantity.note }})
  {%- endif %}
{%- endmacro quantity %}

{% macro basic_table(ingredients) -%}
  {{ lf -}}
  | Name | Quantity |{{ lf -}}
  | --- | --- |{{ lf }}
  {%- for ingredient in ingredients -%}
    | {{ ingredient.name }}
    {%- if ingredient.kind is string -%}
      , {{ ingredient.kind }}
    {%- endif -%}
    {{ " | " }}
    {%- if ingredient.quantity is object -%}
      {{ table::quantity(quantity = ingredient.quantity) }}
    {%- endif %} |{{ lf }}
  {%- endfor %}
{%- endmacro basic_table %}

{% macro sectioned_table(sections) %}
  {%- for section in sections -%}
    {{ lf -}}
    ### {{ section.name ~ lf }}
    {{- table::basic_table(ingredients = section.items) }}
  {%- endfor %}
{%- endmacro sectioned_table %}

{% macro table(list) %}
  {%- if list.sections is defined -%}
    {{ table::sectioned_table(sections = list.sections) }}
  {%- else -%}
    {{ table::basic_table(ingredients = list.items) }}
  {%- endif %}
{%- endmacro table %}

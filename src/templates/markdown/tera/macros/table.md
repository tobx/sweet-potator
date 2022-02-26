{% macro quantity(quantity) %}
  {%- if quantity.value.fraction is defined -%}
    {{ quantity.value.fraction.numerator }}/{{ quantity.value.fraction.denominator }}
  {%- else -%}
    {{ quantity.value.integer }}
  {%- endif %}
  {%- if quantity.unit is string -%}
    {{ " " ~ quantity.unit }}
  {%- endif %}
  {%- if quantity.note is string -%}
    {{ " " }}({{ quantity.note }})
  {%- endif %}
{%- endmacro quantity %}

{% macro basic_table(ingredients) -%}
  {{ lf -}}
  | Quantity | Name |{{ lf -}}
  | --- | --- |{{ lf }}
  {%- for ingredient in ingredients -%}
    | {% if ingredient.quantity is object -%}
      {{ table::quantity(quantity = ingredient.quantity) }}
    {%- endif -%}
    {{ " | " }}
    {{- ingredient.name }}
    {%- if ingredient.kind is string -%}
      , {{ ingredient.kind }}
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

{% macro basic_list(items) %}
  {%- for item in items %}
    {%- if loop.first %}{{ lf }}{% endif -%}
    {{ loop.index }}. {{ item ~ lf }}
  {%- endfor %}
{%- endmacro list %}

{% macro sectioned_list(sections) %}
  {%- for section in sections -%}
    {{ lf }}### {{ section.name ~ lf }}
    {{- self::basic_list(items = section.items) }}
  {%- endfor %}
{%- endmacro sectioned_list %}

{% macro list(list) %}
  {%- if list.sections is defined -%}
    {{ self::sectioned_list(sections = list.sections) }}
  {%- else -%}
    {{ self::basic_list(items = list.items) }}
  {%- endif %}
{%- endmacro list %}

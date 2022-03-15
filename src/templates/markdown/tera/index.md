{% extends "blocks/base.md" %}

{% block main -%}
# {{ lang.index.heading }}

{% for recipe in recipes -%}
  - [{{ recipe.title }}]({{ recipe.path | escape_xml | safe }})
{% endfor %}
{%- endblock main %}

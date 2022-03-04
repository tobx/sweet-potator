{% extends "blocks/base.md" %}

{% block main -%}
# Recipes

{% for recipe in recipes -%}
  - [{{ recipe.title }}]({{ recipe.path | escape_xml | safe }})
{% endfor %}
{%- endblock main %}

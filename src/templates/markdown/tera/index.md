{% extends "blocks/base.md" %}

{% block main -%}
# Recipes

{% for recipe in recipes -%}
  - [{{ recipe.title }}]({{ recipe.path | urlencode | safe }})
{% endfor %}
{%- endblock main %}

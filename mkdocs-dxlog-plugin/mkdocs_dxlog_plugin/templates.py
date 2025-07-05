"""Default templates for dxlog research items."""

from typing import Dict, Any


class DefaultTemplates:
    """Default templates for different research item types."""
    
    @staticmethod
    def get_hypothesis_template() -> str:
        """Get default template for hypothesis items."""
        return """# {{ title }}

## Hypothesis

{{ metadata.hypothesis | default("No hypothesis defined") }}

## Status

**Current Status**: {{ metadata.status | default("active") }}
**Confidence**: {{ metadata.confidence | default("unknown") }}
**Date**: {{ metadata.date | default("not specified") }}

## Evidence

{{ metadata.evidence | default("No evidence provided") }}

## Content

{{ content }}

{% if tags %}
## Tags
{% for tag in tags %}
- {{ tag }}
{% endfor %}
{% endif %}

{% if related_items %}
## Related Items
{% for item in related_items %}
- [{{ item.title }}]({{ item.link }})
{% endfor %}
{% endif %}

{% if backlinks %}
## Referenced By
{% for item in backlinks %}
- [{{ item.title }}]({{ item.link }})
{% endfor %}
{% endif %}
"""
    
    @staticmethod
    def get_literature_template() -> str:
        """Get default template for literature items."""
        return """# {{ title }}

## Publication Details

**Authors**: {{ metadata.authors | default("Unknown") }}
**Journal**: {{ metadata.journal | default("Unknown") }}
**Year**: {{ metadata.year | default("Unknown") }}
**DOI**: {{ metadata.doi | default("Not available") }}
**URL**: {{ metadata.url | default("Not available") }}

## Status

**Review Status**: {{ metadata.status | default("active") }}
**Date Added**: {{ metadata.date | default("not specified") }}

## Summary

{{ metadata.summary | default("No summary provided") }}

## Content

{{ content }}

{% if metadata.keywords %}
## Keywords
{% for keyword in metadata.keywords %}
- {{ keyword }}
{% endfor %}
{% endif %}

{% if tags %}
## Tags
{% for tag in tags %}
- {{ tag }}
{% endfor %}
{% endif %}

{% if related_items %}
## Related Items
{% for item in related_items %}
- [{{ item.title }}]({{ item.link }})
{% endfor %}
{% endif %}

{% if backlinks %}
## Referenced By
{% for item in backlinks %}
- [{{ item.title }}]({{ item.link }})
{% endfor %}
{% endif %}
"""
    
    @staticmethod
    def get_knowledge_template() -> str:
        """Get default template for knowledge base items."""
        return """# {{ title }}

## Knowledge Entry

**Type**: {{ metadata.type | default("knowledge") }}
**Status**: {{ metadata.status | default("active") }}
**Date**: {{ metadata.date | default("not specified") }}

## Summary

{{ metadata.summary | default("No summary provided") }}

## Content

{{ content }}

{% if metadata.source %}
## Source

{{ metadata.source }}
{% endif %}

{% if metadata.keywords %}
## Keywords
{% for keyword in metadata.keywords %}
- {{ keyword }}
{% endfor %}
{% endif %}

{% if tags %}
## Tags
{% for tag in tags %}
- {{ tag }}
{% endfor %}
{% endif %}

{% if related_items %}
## Related Items
{% for item in related_items %}
- [{{ item.title }}]({{ item.link }})
{% endfor %}
{% endif %}

{% if backlinks %}
## Referenced By
{% for item in backlinks %}
- [{{ item.title }}]({{ item.link }})
{% endfor %}
{% endif %}
"""
    
    @staticmethod
    def get_overview_template() -> str:
        """Get default template for overview pages."""
        return """# {{ item_type.title() }} Overview

This section contains {{ total_items }} {{ item_type }} items.

{% for status, items in items_by_status.items() %}
## {{ status.title() }} ({{ items|length }} items)

{% for item in items %}
- [{{ item.title }}]({{ item.link }}){% if item.date %} - {{ item.date }}{% endif %}
{% endfor %}

{% endfor %}

## Statistics

- **Total Items**: {{ total_items }}
- **Active Items**: {{ stats.active | default(0) }}
- **Completed Items**: {{ stats.completed | default(0) }}
- **Archived Items**: {{ stats.archived | default(0) }}
"""
    
    @staticmethod
    def get_template_for_type(item_type: str) -> str:
        """Get template for a specific item type."""
        templates = {
            "hypothesis": DefaultTemplates.get_hypothesis_template(),
            "literature": DefaultTemplates.get_literature_template(),
            "knowledge": DefaultTemplates.get_knowledge_template(),
            "overview": DefaultTemplates.get_overview_template()
        }
        
        return templates.get(item_type, DefaultTemplates.get_hypothesis_template())
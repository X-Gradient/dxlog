"""Content generation system for dxlog MkDocs integration."""

import os
import re
from typing import Dict, List, Any, Optional
from pathlib import Path
import logging

from mkdocs.structure.files import File
from mkdocs.structure.nav import Section
from mkdocs.structure.pages import Page

logger = logging.getLogger(__name__)


class ContentGenerator:
    """Generates MkDocs content from dxlog research items."""
    
    def __init__(self, section_name: str, include_types: List[str], show_archived: bool, 
                 landing_config: Optional[Dict[str, Any]] = None):
        self.section_name = section_name
        self.include_types = include_types
        self.show_archived = show_archived
        self.landing_config = landing_config or {
            'show_statistics': True,
            'show_recent_activity': True,
            'show_items_by_type': True,
            'show_items_by_status': True,
            'show_quick_navigation': True,
            'recent_items_limit': 5,
            'items_per_section_limit': 5,
            'show_table_view': True,
            'table_sort_by': 'date',
            'table_sort_order': 'desc',
            'max_table_rows': 100
        }
        
    def get_file_path(self, item: Dict[str, Any]) -> str:
        """Generate file path for a research item."""
        item_type = item.get("type", "unknown")
        category = item.get("category", "active")
        title = item.get("title", "untitled")
        
        # Sanitize title for filename
        safe_title = re.sub(r'[^\w\s-]', '', title).strip()
        safe_title = re.sub(r'[-\s]+', '-', safe_title)
        
        # Build path
        path_parts = [self.section_name.lower()]
        
        if category == "archived" and self.show_archived:
            path_parts.append("archived")
        
        path_parts.extend([item_type, f"{safe_title}.md"])
        
        return "/".join(path_parts)
    
    def generate_page_content(self, item: Dict[str, Any], item_id: str, 
                            cross_references: Dict[str, List[str]], all_items: Dict[str, Dict[str, Any]] = None) -> str:
        """Generate markdown content for a research item page."""
        metadata = item.get("metadata", {})
        content = item.get("content", "")
        
        # Build page header
        title = item.get("title", "Untitled")
        item_type = item.get("type", "unknown").title()
        
        page_content = []
        
        # Add main content
        if content.strip():
            page_content.append(content)
        
        # Add enhanced cross-references BEFORE the metadata section
        refs = cross_references.get(item_id, [])
        if refs:
            page_content.append("\n## Related Items\n")
            for ref_id in refs:
                ref_link = self._generate_enhanced_reference_link(ref_id, all_items)
                page_content.append(f"- {ref_link}")
        
        # Add enhanced backlinks
        backlinks = self._find_backlinks(item_id, cross_references)
        if backlinks:
            page_content.append("\n## Referenced By\n")
            for ref_id in backlinks:
                ref_link = self._generate_enhanced_reference_link(ref_id, all_items)
                page_content.append(f"- {ref_link}")
        
        page_content.append("\n---")

        # Add simplified metadata
        metadata_section = self._generate_metadata_card(item, metadata, item_type)
        page_content.append(metadata_section)
        
        # Add literature source buttons if applicable
        if item_type.lower() == "literature" and "source" in metadata:
            source_buttons = self._generate_source_buttons(metadata["source"])
            if source_buttons:
                page_content.append(source_buttons)
        
        return "\n".join(page_content)
    
    def _get_custom_fields(self, metadata: Dict[str, Any], item_type: str) -> Dict[str, Any]:
        """Extract custom fields based on item type."""
        # Common fields to exclude
        common_fields = {"title", "date", "status", "tags", "uuid", "id", "type"}
        
        # Type-specific fields
        type_fields = {
            "hypothesis": {"hypothesis", "evidence", "confidence"},
            "literature": {"authors", "journal", "year", "doi", "url"},
            "knowledge": {"summary", "keywords", "source"}
        }
        
        excluded_fields = common_fields | type_fields.get(item_type, set())
        
        custom_fields = {}
        for key, value in metadata.items():
            if key not in excluded_fields and value is not None:
                custom_fields[key] = value
        
        return custom_fields
    
    def _generate_internal_link(self, item_id: str, all_items: Dict[str, Dict[str, Any]] = None) -> str:
        """Generate MkDocs internal link for an item."""
        if all_items and item_id in all_items:
            item = all_items[item_id]
            # Get the item type and generate relative path
            item_type = item.get("type", "unknown")
            title = item.get("title", "untitled")
            
            # Sanitize title for URL
            safe_title = re.sub(r'[^\w\s-]', '', title).strip()
            safe_title = re.sub(r'[-\s]+', '-', safe_title)
            
            # Generate relative path from current item to target item
            # Current path: mcp research/current_type/current_item/
            # Target path: mcp research/target_type/target_item/
            # Relative path: ../../target_type/target_item/
            return f"../../{item_type}/{safe_title}/"
        else:
            # Fallback to simple approach
            return f"../{item_id}/"
    
    def _generate_metadata_card(self, item: Dict[str, Any], metadata: Dict[str, Any], item_type: str) -> str:
        """Generate simplified metadata display."""
        content = []
        
        # Simple metadata section
        content.append('\n<div class="dxlog-metadata-simple" markdown="1">')
        
        metadata_content = []
        # Type and Status on same line
        type_badge = self._get_type_badge(item_type.lower())
        status_info = ""
        if "status" in metadata:
            status = metadata["status"]
            status_badge = self._get_status_badge(status)
            status_class = self._get_status_class(status)
            status_info = f' • <span class="dxlog-status {status_class}">{status_badge} {status}</span>'
        
        metadata_content.append(f'**Type**: {type_badge} {item_type}{status_info}')
        
        # Date
        if "date" in metadata:
            date_str = metadata["date"]
            formatted_date = self._format_date(date_str)
            metadata_content.append(f'**Date**: {formatted_date}')
        
        # Creator
        if "created_by" in metadata:
            creator = metadata["created_by"]
            if isinstance(creator, dict):
                name = creator.get("name", "Unknown")
                email = creator.get("email", "")
                creator_text = name
                if email:
                    creator_text += f" ({email})"
                metadata_content.append(f'**Author**: {creator_text}')
        
        # Tags
        if "tags" in metadata:
            tags = metadata["tags"]
            if isinstance(tags, list) and tags:
                tag_pills = []
                for tag in tags:
                    tag_class = self._get_tag_class(tag)
                    tag_pills.append(f'<span class="dxlog-tag {tag_class}">{tag}</span>')
                metadata_content.append(f'**Tags**: {" ".join(tag_pills)}')
        
        content.append("\n<br>".join(metadata_content))
        content.append('</div>')

        return "\n".join(content)
    
    def _generate_source_buttons(self, source: Dict[str, Any]) -> str:
        """Generate action buttons for literature sources."""
        if not source:
            return ""
        
        buttons = []
        button_content = ['\n<div class="dxlog-source-buttons" markdown="1">']
        button_content.append('## 🔗 External Sources\n')
        
        # arXiv button
        if source.get("arxiv_url"):
            arxiv_url = source["arxiv_url"]
            buttons.append(f'<a href="{arxiv_url}" class="dxlog-source-btn dxlog-arxiv-btn" target="_blank">📄 arXiv Paper</a>')
        
        # GitHub button
        if source.get("repository_url"):
            repo_url = source["repository_url"]
            buttons.append(f'<a href="{repo_url}" class="dxlog-source-btn dxlog-github-btn" target="_blank">🐙 GitHub Repository</a>')
        
        # DOI button
        if source.get("doi"):
            doi = source["doi"]
            doi_url = f"https://doi.org/{doi}" if not doi.startswith("http") else doi
            buttons.append(f'<a href="{doi_url}" class="dxlog-source-btn dxlog-doi-btn" target="_blank">🎓 DOI Link</a>')
        
        # PDF button
        if source.get("pdf_url"):
            pdf_url = source["pdf_url"]
            buttons.append(f'<a href="{pdf_url}" class="dxlog-source-btn dxlog-pdf-btn" target="_blank">📑 PDF Download</a>')
        
        if buttons:
            button_content.append('<div class="dxlog-button-group">')
            button_content.extend(buttons)
            button_content.append('</div>')
            button_content.append('</div>')
            return "\n".join(button_content)
        
        return ""
    
    def _generate_enhanced_reference_link(self, ref_id: str, all_items: Dict[str, Dict[str, Any]] = None) -> str:
        """Generate enhanced reference link with title resolution."""
        # Try to find the referenced item to get its title
        ref_title = self._resolve_reference_title(ref_id, all_items)
        if ref_title:
            link = self._generate_internal_link(ref_id, all_items)
            ref_type = self._get_reference_type(ref_id, all_items)
            type_badge = self._get_type_badge(ref_type) if ref_type else ""
            return f"{type_badge} [{ref_title}]({link}) <small>({ref_id[:8]}...)</small>"
        else:
            link = self._generate_internal_link(ref_id, all_items)
            return f"[{ref_id}]({link})"
    
    def _resolve_reference_title(self, ref_id: str, all_items: Dict[str, Dict[str, Any]] = None) -> str:
        """Resolve reference ID to actual title."""
        if all_items and ref_id in all_items:
            return all_items[ref_id].get("title", ref_id)
        return None
    
    def _get_reference_type(self, ref_id: str, all_items: Dict[str, Dict[str, Any]] = None) -> str:
        """Get the type of the referenced item."""
        if all_items and ref_id in all_items:
            return all_items[ref_id].get("type", "unknown")
        return "unknown"
    
    def _format_date(self, date_str: str) -> str:
        """Format date string for better display."""
        if not date_str:
            return "Not specified"
        
        try:
            from datetime import datetime
            date_obj = datetime.strptime(date_str, "%Y-%m-%d")
            return date_obj.strftime("%B %d, %Y")
        except:
            return date_str
    
    def _generate_avatar(self, name: str) -> str:
        """Generate a simple avatar for the creator."""
        if not name:
            return "👤"
        
        # Generate initials
        initials = "".join([word[0].upper() for word in name.split() if word][:2])
        return f'<span class="dxlog-avatar">{initials}</span>'
    
    def _get_status_class(self, status: str) -> str:
        """Get CSS class for status styling."""
        status_lower = status.lower()
        class_map = {
            'active': 'dxlog-status-active',
            'completed': 'dxlog-status-completed',
            'proven': 'dxlog-status-proven',
            'disproven': 'dxlog-status-disproven',
            'archived': 'dxlog-status-archived',
            'inprogress': 'dxlog-status-inprogress',
            'draft': 'dxlog-status-draft',
            'published': 'dxlog-status-published'
        }
        return class_map.get(status_lower, 'dxlog-status-default')
    
    def _get_tag_class(self, tag: str) -> str:
        """Get CSS class for tag styling based on tag content."""
        tag_lower = tag.lower()
        
        # Category-based styling
        if tag_lower in ['security', 'vulnerability', 'threat']:
            return 'dxlog-tag-security'
        elif tag_lower in ['arxiv', 'github', 'academic']:
            return 'dxlog-tag-source'
        elif tag_lower in ['implementation', 'code', 'technical']:
            return 'dxlog-tag-technical'
        elif tag_lower in ['enterprise', 'business', 'industry']:
            return 'dxlog-tag-business'
        else:
            return 'dxlog-tag-default'
    
    def _find_backlinks(self, item_id: str, cross_references: Dict[str, List[str]]) -> List[str]:
        """Find items that reference this item."""
        backlinks = []
        for ref_item_id, refs in cross_references.items():
            if item_id in refs:
                backlinks.append(ref_item_id)
        return backlinks
    
    def create_overview_files(self, items: Dict[str, Dict[str, Any]], 
                            config: Dict[str, Any]) -> List[File]:
        """Create overview pages for each research item type."""
        overview_files = []
        
        # Create main landing page
        main_landing_file = self._create_main_landing_file(items, config)
        if main_landing_file:
            overview_files.append(main_landing_file)
            logger.info(f"Added main landing file to overview_files list")
        else:
            logger.error("Failed to create main landing file!")
        
        # Group items by type
        items_by_type = {}
        for item_id, item in items.items():
            item_type = item.get("type", "unknown")
            if item_type not in items_by_type:
                items_by_type[item_type] = []
            items_by_type[item_type].append((item_id, item))
        
        # Create overview file for each type
        for item_type, type_items in items_by_type.items():
            if item_type not in self.include_types:
                continue
                
            file_path = f"{self.section_name.lower()}/{item_type}/index.md"
            
            virtual_file = File(
                path=file_path,
                src_dir=config['docs_dir'],
                dest_dir=config['site_dir'],
                use_directory_urls=config['use_directory_urls']
            )
            
            # Generate overview content
            virtual_file.dxlog_overview = {
                "type": item_type,
                "items": type_items
            }
            
            overview_files.append(virtual_file)
        
        return overview_files
    
    def _create_main_landing_file(self, items: Dict[str, Dict[str, Any]], 
                                 config: Dict[str, Any]) -> Optional[File]:
        """Create main landing page for dxlog section."""
        try:
            # Use the section name + index pattern that works with mkdocs-section-index
            file_path = f"{self.section_name.lower()}/index.md"
            logger.info(f"Creating main landing file at path: {file_path}")
            logger.info(f"Items count: {len(items)}")
            
            # Create virtual file for the landing page
            virtual_file = File(
                path=file_path,
                src_dir=config['docs_dir'],
                dest_dir=config['site_dir'],
                use_directory_urls=config['use_directory_urls']
            )
            
            # Mark this as a section index file for mkdocs-section-index plugin
            # Don't set virtual_file.name here - let navigation set it
            virtual_file.is_section_index = True  # Custom attribute for our plugin
            
            logger.info(f"Virtual file created with src_path: {virtual_file.src_path}")
            logger.info(f"Virtual file abs_src_path: {virtual_file.abs_src_path}")
            
            # Generate landing page content
            virtual_file.dxlog_main_landing = {
                "section_name": self.section_name,
                "items": items,
                "include_types": self.include_types,
                "show_archived": self.show_archived
            }
            
            logger.info(f"Successfully created main landing file")
            return virtual_file
            
        except Exception as e:
            logger.error(f"Error creating main landing file: {e}")
            return None
    
    def generate_overview_content(self, overview_data: Dict[str, Any]) -> str:
        """Generate content for overview pages."""
        item_type = overview_data["type"]
        items = overview_data["items"]
        
        content = [f"# {item_type.title()} Overview"]
        content.append(f"\nThis section contains {len(items)} {item_type} items.\n")
        
        # Group by status
        items_by_status = {}
        for item_id, item in items:
            status = item.get("metadata", {}).get("status", "unknown")
            if status not in items_by_status:
                items_by_status[status] = []
            items_by_status[status].append((item_id, item))
        
        # List items by status
        for status, status_items in items_by_status.items():
            content.append(f"## {status.title()} ({len(status_items)} items)")
            
            for item_id, item in status_items:
                title = item.get("title", item_id)
                link = self._generate_internal_link(item_id)
                date = item.get("metadata", {}).get("date", "")
                
                if date:
                    content.append(f"- [{title}]({link}) - {date}")
                else:
                    content.append(f"- [{title}]({link})")
        
        return "\n".join(content)
    
    def generate_main_landing_content(self, landing_data: Dict[str, Any]) -> str:
        """Generate comprehensive main landing page content."""
        # Check if table view is enabled
        if self.landing_config.get('show_table_view', True):
            return self.generate_table_landing_content(landing_data)
        
        # Original grouped view
        section_name = landing_data["section_name"]
        items = landing_data["items"]
        include_types = landing_data["include_types"]
        show_archived = landing_data["show_archived"]
        
        content = []
        
        # Header
        content.append(f"# {section_name}")
        content.append(f"\nWelcome to your {section_name.lower()} documentation. This section contains all your research items from your dxlog project.\n")
        
        # Statistics (configurable)
        if self.landing_config.get('show_statistics', True):
            stats = self._calculate_statistics(items, include_types, show_archived)
            content.append("## 📊 Project Statistics\n")
            content.append(f"- **Total Items**: {stats['total']}")
            content.append(f"- **Active Items**: {stats['active']}")
            content.append(f"- **Completed Items**: {stats['completed']}")
            content.append(f"- **Archived Items**: {stats['archived']}")
            
            # Breakdown by type
            content.append("\n### By Type\n")
            for item_type in include_types:
                count = stats['by_type'].get(item_type, 0)
                content.append(f"- **{item_type.title()}**: {count}")
        
        # Recent activity (configurable)
        if self.landing_config.get('show_recent_activity', True):
            recent_items_limit = self.landing_config.get('recent_items_limit', 5)
            recent_items = self._get_recent_items(items, recent_items_limit)
            if recent_items:
                content.append("\n## 🕒 Recent Activity\n")
                for item_id, item in recent_items:
                    title = item.get("title", item_id)
                    item_type = item.get("type", "unknown")
                    status = item.get("metadata", {}).get("status", "unknown")
                    date = item.get("metadata", {}).get("date", "")
                    
                    # Generate link
                    link_path = self.get_file_path(item)
                    link = f"../{link_path}".replace(".md", "/")
                    
                    date_str = f" - {date}" if date else ""
                    content.append(f"- [{title}]({link}) *({item_type}, {status})*{date_str}")
        
        # Items by type (configurable)
        if self.landing_config.get('show_items_by_type', True):
            items_per_section_limit = self.landing_config.get('items_per_section_limit', 5)
            content.append("\n## 📚 Research Items by Type\n")
            items_by_type = self._group_items_by_type(items, include_types)
            
            for item_type in include_types:
                if item_type not in items_by_type:
                    continue
                    
                type_items = items_by_type[item_type]
                content.append(f"\n### {item_type.title()} ({len(type_items)} items)\n")
                
                # Link to type overview
                type_overview_link = f"./{item_type}/"
                content.append(f"[View all {item_type} items]({type_overview_link})\n")
                
                # Show first few items
                for item_id, item in type_items[:items_per_section_limit]:
                    title = item.get("title", item_id)
                    status = item.get("metadata", {}).get("status", "unknown")
                    date = item.get("metadata", {}).get("date", "")
                    
                    # Generate link
                    link_path = self.get_file_path(item)
                    link = f"../{link_path}".replace(".md", "/")
                    
                    status_badge = self._get_status_badge(status)
                    date_str = f" - {date}" if date else ""
                    content.append(f"- [{title}]({link}) {status_badge}{date_str}")
                
                if len(type_items) > items_per_section_limit:
                    content.append(f"- ... and {len(type_items) - items_per_section_limit} more items")
        
        # Items by status (configurable)
        if self.landing_config.get('show_items_by_status', True):
            content.append("\n## 📋 Research Items by Status\n")
            items_by_status = self._group_items_by_status(items, show_archived)
            
            for status in ['active', 'completed', 'archived']:
                if status not in items_by_status:
                    continue
                if status == 'archived' and not show_archived:
                    continue
                    
                status_items = items_by_status[status]
                content.append(f"\n### {status.title()} ({len(status_items)} items)\n")
                
                for item_id, item in status_items[:10]:  # Show first 10 items
                    title = item.get("title", item_id)
                    item_type = item.get("type", "unknown")
                    date = item.get("metadata", {}).get("date", "")
                    
                    # Generate link
                    link_path = self.get_file_path(item)
                    link = f"../{link_path}".replace(".md", "/")
                    
                    type_badge = self._get_type_badge(item_type)
                    date_str = f" - {date}" if date else ""
                    content.append(f"- [{title}]({link}) {type_badge}{date_str}")
                
                if len(status_items) > 10:
                    content.append(f"- ... and {len(status_items) - 10} more items")
        
        # Quick navigation (configurable)
        if self.landing_config.get('show_quick_navigation', True):
            content.append("\n## 🧭 Quick Navigation\n")
            for item_type in include_types:
                type_overview_link = f"./{item_type}/"
                content.append(f"- [{item_type.title()}]({type_overview_link}) - View all {item_type} items")
        
        return "\n".join(content)
    
    def generate_table_landing_content(self, landing_data: Dict[str, Any]) -> str:
        """Generate table-based main landing page content."""
        section_name = landing_data["section_name"]
        items = landing_data["items"]
        include_types = landing_data["include_types"]
        show_archived = landing_data["show_archived"]
        
        content = []
        
        # Header
        content.append(f"# {section_name}")
        content.append(f"\nYour {section_name.lower()} documentation with all research items.\n")
        
        # Filter and prepare items for table
        table_items = self._prepare_table_items(items, include_types, show_archived)
        
        if not table_items:
            content.append("No research items found.")
            return "\n".join(content)
        
        # Sort items
        sorted_items = self._sort_table_items(table_items)
        
        # Limit items if configured
        max_rows = self.landing_config.get('max_table_rows', 100)
        if len(sorted_items) > max_rows:
            sorted_items = sorted_items[:max_rows]
            content.append(f"*Showing {max_rows} of {len(table_items)} items*\n")
        
        # Generate table
        table_content = self._generate_markdown_table(sorted_items)
        content.append(table_content)
        
        # Add summary stats
        content.append(f"\n**Total Items**: {len(table_items)}")
        
        return "\n".join(content)
    
    def _prepare_table_items(self, items: Dict[str, Dict[str, Any]], 
                           include_types: List[str], show_archived: bool) -> List[Dict[str, Any]]:
        """Prepare items for table display."""
        table_items = []
        
        for item_id, item in items.items():
            item_type = item.get("type", "unknown")
            category = item.get("category", "active")
            
            # Filter by type
            if item_type not in include_types:
                continue
            
            # Filter archived items if not shown
            if category == "archived" and not show_archived:
                continue
            
            metadata = item.get("metadata", {})
            
            # Generate link
            link_path = self.get_file_path(item)
            link = f"../{link_path}".replace(".md", "/")
            
            table_item = {
                'id': item_id,
                'title': item.get("title", item_id),
                'type': item_type,
                'status': metadata.get("status", "active"),
                'date': metadata.get("date", ""),
                'link': link,
                'category': category
            }
            
            table_items.append(table_item)
        
        return table_items
    
    def _sort_table_items(self, items: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Sort table items based on configuration."""
        sort_by = self.landing_config.get('table_sort_by', 'date')
        sort_order = self.landing_config.get('table_sort_order', 'desc')
        reverse = sort_order.lower() == 'desc'
        
        if sort_by == 'name':
            return sorted(items, key=lambda x: x['title'].lower(), reverse=reverse)
        elif sort_by == 'type':
            return sorted(items, key=lambda x: x['type'], reverse=reverse)
        elif sort_by == 'status':
            return sorted(items, key=lambda x: x['status'], reverse=reverse)
        elif sort_by == 'date':
            # Sort by date, treating empty dates as very old
            return sorted(items, key=lambda x: x['date'] if x['date'] else '1900-01-01', reverse=reverse)
        else:
            return items
    
    def _generate_markdown_table(self, items: List[Dict[str, Any]]) -> str:
        """Generate markdown table from items."""
        if not items:
            return "No items to display."
        
        lines = []
        
        # Table header
        lines.append("| Name | Type | Status | Date |")
        lines.append("|------|------|--------|------|")
        
        # Table rows
        for item in items:
            title = item['title']
            link = item['link']
            item_type = item['type']
            status = item['status']
            date = item['date'] if item['date'] else 'N/A'
            
            # Get badges
            type_badge = self._get_type_badge(item_type)
            status_badge = self._get_status_badge(status)
            
            # Create table row
            name_cell = f"[{title}]({link})"
            type_cell = f"{type_badge} {item_type.title()}"
            status_cell = f"{status_badge} {status.title()}"
            date_cell = date
            
            lines.append(f"| {name_cell} | {type_cell} | {status_cell} | {date_cell} |")
        
        return "\n".join(lines)
    
    def _calculate_statistics(self, items: Dict[str, Dict[str, Any]], 
                             include_types: List[str], show_archived: bool) -> Dict[str, Any]:
        """Calculate statistics for the landing page."""
        stats = {
            'total': 0,
            'active': 0,
            'completed': 0,
            'archived': 0,
            'by_type': {}
        }
        
        for item_id, item in items.items():
            item_type = item.get("type", "unknown")
            if item_type not in include_types:
                continue
                
            status = item.get("metadata", {}).get("status", "active")
            category = item.get("category", "active")
            
            # Skip archived items if not shown
            if category == "archived" and not show_archived:
                continue
            
            stats['total'] += 1
            
            # Count by status - map dxlog statuses to logical groups for stats
            normalized_status = self._normalize_status_for_stats(status, category)
            if normalized_status in stats:
                stats[normalized_status] += 1
            
            # Count by type
            if item_type not in stats['by_type']:
                stats['by_type'][item_type] = 0
            stats['by_type'][item_type] += 1
        
        return stats
    
    def _get_recent_items(self, items: Dict[str, Dict[str, Any]], 
                         limit: int = 5) -> List[tuple]:
        """Get most recent items based on date."""
        item_list = []
        for item_id, item in items.items():
            date = item.get("metadata", {}).get("date", "")
            item_list.append((item_id, item, date))
        
        # Sort by date (newest first)
        item_list.sort(key=lambda x: x[2], reverse=True)
        
        return [(item_id, item) for item_id, item, _ in item_list[:limit]]
    
    def _group_items_by_type(self, items: Dict[str, Dict[str, Any]], 
                            include_types: List[str]) -> Dict[str, List[tuple]]:
        """Group items by type."""
        items_by_type = {}
        for item_id, item in items.items():
            item_type = item.get("type", "unknown")
            if item_type in include_types:
                if item_type not in items_by_type:
                    items_by_type[item_type] = []
                items_by_type[item_type].append((item_id, item))
        
        # Sort each type by date
        for item_type in items_by_type:
            items_by_type[item_type].sort(
                key=lambda x: x[1].get("metadata", {}).get("date", ""), 
                reverse=True
            )
        
        return items_by_type
    
    def _group_items_by_status(self, items: Dict[str, Dict[str, Any]], 
                              show_archived: bool) -> Dict[str, List[tuple]]:
        """Group items by status."""
        items_by_status = {}
        for item_id, item in items.items():
            status = item.get("metadata", {}).get("status", "active")
            category = item.get("category", "active")
            
            # Map category to status if needed
            if category == "archived":
                status = "archived"
            # Keep original dxlog status values instead of mapping to "active"
            
            # Skip archived items if not shown
            if status == "archived" and not show_archived:
                continue
            
            if status not in items_by_status:
                items_by_status[status] = []
            items_by_status[status].append((item_id, item))
        
        # Sort each status by date
        for status in items_by_status:
            items_by_status[status].sort(
                key=lambda x: x[1].get("metadata", {}).get("date", ""), 
                reverse=True
            )
        
        return items_by_status
    
    def _get_status_badge(self, status: str) -> str:
        """Get a status badge for display."""
        # Handle all valid dxlog status values
        badge_map = {
            # Original statuses
            'active': '🟢',
            'completed': '✅',
            'archived': '📦',
            
            # dxlog hypothesis statuses
            'proven': '✅',
            'disproven': '❌',
            'inconclusive': '❓',
            'suspended': '⏸️',
            'abandoned': '🗑️',
            
            # Handle case variations
            'Active': '🟢',
            'Proven': '✅',
            'Disproven': '❌',
            'Inconclusive': '❓',
            'Suspended': '⏸️',
            'Abandoned': '🗑️'
        }
        return badge_map.get(status, '⚪')
    
    def _normalize_status_for_stats(self, status: str, category: str) -> str:
        """Normalize dxlog statuses to logical groups for statistics."""
        # Handle case variations
        status_lower = status.lower()
        
        if category == "archived":
            return "archived"
        
        # Map dxlog statuses to logical groups
        if status_lower in ['active']:
            return 'active'
        elif status_lower in ['proven', 'completed']:
            return 'completed'
        elif status_lower in ['disproven', 'inconclusive', 'suspended', 'abandoned']:
            return 'completed'  # Treat as completed research even if not proven
        else:
            return 'active'  # Default fallback
    
    def _get_type_badge(self, item_type: str) -> str:
        """Get a type badge for display."""
        badge_map = {
            'hypothesis': '🔬',
            'literature': '📚',
            'knowledge': '💡'
        }
        return badge_map.get(item_type, '📄')
    
    def generate_navigation(self, items: Dict[str, Dict[str, Any]], 
                          files) -> Optional[Section]:
        """Generate navigation structure for dxlog content."""
        try:
            # Find the main landing page first
            main_landing_file = self._find_main_landing_file(files)
            
            # Create main section
            section = Section(self.section_name, [])
            
            # Add main landing page as the first item with "Overview" title
            if main_landing_file:
                # Set the name to "Overview" to make it clear this is the overview page
                main_landing_file.name = "Overview"
                section.children.append(main_landing_file)
                logger.info(f"Added main landing page to navigation: {main_landing_file.src_path}")
                logger.info(f"Landing page will be accessible at: {main_landing_file.url}")
            else:
                logger.warning("Main landing file not found in navigation generation")
            
            # Group items by type
            items_by_type = {}
            for item_id, item in items.items():
                item_type = item.get("type", "unknown")
                if item_type in self.include_types:
                    if item_type not in items_by_type:
                        items_by_type[item_type] = []
                    items_by_type[item_type].append((item_id, item))
            
            # Create navigation for each type as subsections
            for item_type in self.include_types:
                if item_type not in items_by_type:
                    continue
                
                type_section = Section(item_type.title(), [])
                
                # Add overview page for this type
                overview_file = self._find_overview_file(item_type, files)
                if overview_file:
                    overview_file.name = "Overview"  # Make it clear this is the overview
                    type_section.children.append(overview_file)
                
                # Add individual items
                for item_id, item in items_by_type[item_type]:
                    item_file = self._find_item_file(item_id, files)
                    if item_file:
                        # Set a proper title for the navigation
                        item_title = item.get("title", item_id)
                        item_file.name = item_title
                        type_section.children.append(item_file)
                
                section.children.append(type_section)
            
            logger.info(f"Generated navigation with {len(section.children)} top-level items")
            return section
            
        except Exception as e:
            logger.error(f"Error generating navigation: {e}")
            return None
    
    def _find_overview_file(self, item_type: str, files) -> Optional[File]:
        """Find overview file for a given item type."""
        overview_path = f"{self.section_name.lower()}/{item_type}/index.md"
        
        for file in files:
            if file.src_path == overview_path:
                return file
        
        return None
    
    def _find_main_landing_file(self, files) -> Optional[File]:
        """Find main landing file for dxlog section."""
        landing_path = f"{self.section_name.lower()}/index.md"
        
        for file in files:
            if file.src_path == landing_path:
                return file
        
        return None
    
    def _find_item_file(self, item_id: str, files) -> Optional[File]:
        """Find file for a given item ID."""
        for file in files:
            if hasattr(file, 'dxlog_id') and file.dxlog_id == item_id:
                return file
        
        return None
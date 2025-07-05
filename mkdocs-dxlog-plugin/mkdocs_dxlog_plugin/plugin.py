"""MkDocs plugin for dxlog research project integration."""

import os
import logging
from typing import Any, Dict, List, Optional

from mkdocs.config import config_options
from mkdocs.plugins import BasePlugin
from mkdocs.structure.files import File
from mkdocs.structure.nav import Navigation
from mkdocs.structure.pages import Page

from .dxlog_parser import DxlogParser
from .content_generator import ContentGenerator


logger = logging.getLogger(__name__)


class DxlogPluginConfig(config_options.Config):
    """Configuration for the dxlog plugin."""
    
    dxlog_dir = config_options.Type(str, default=".")
    section_name = config_options.Type(str, default="Research")
    include_types = config_options.Type(list, default=["hypothesis", "literature", "knowledge"])
    show_archived = config_options.Type(bool, default=False)
    
    # Landing page customization options
    show_statistics = config_options.Type(bool, default=True)
    show_recent_activity = config_options.Type(bool, default=True)
    show_items_by_type = config_options.Type(bool, default=True)
    show_items_by_status = config_options.Type(bool, default=True)
    show_quick_navigation = config_options.Type(bool, default=True)
    recent_items_limit = config_options.Type(int, default=5)
    items_per_section_limit = config_options.Type(int, default=5)
    
    # Table view options
    show_table_view = config_options.Type(bool, default=True)
    table_sort_by = config_options.Type(str, default="date")
    table_sort_order = config_options.Type(str, default="desc")
    max_table_rows = config_options.Type(int, default=100)


class DxlogPlugin(BasePlugin[DxlogPluginConfig]):
    """MkDocs plugin for dxlog research project integration."""
    
    def __init__(self):
        super().__init__()
        self.parser = None
        self.content_generator = None
        self.research_items = {}
        self.cross_references = {}
    
    def on_config(self, config: Dict[str, Any]) -> Dict[str, Any]:
        """Initialize the plugin with configuration."""
        try:
            dxlog_dir = os.path.abspath(self.config['dxlog_dir'])
            
            if not os.path.exists(dxlog_dir):
                logger.warning(f"dxlog directory not found: {dxlog_dir}")
                return config
            
            self.parser = DxlogParser(dxlog_dir)
            self.content_generator = ContentGenerator(
                section_name=self.config['section_name'],
                include_types=self.config['include_types'],
                show_archived=self.config['show_archived'],
                landing_config={
                    'show_statistics': self.config['show_statistics'],
                    'show_recent_activity': self.config['show_recent_activity'],
                    'show_items_by_type': self.config['show_items_by_type'],
                    'show_items_by_status': self.config['show_items_by_status'],
                    'show_quick_navigation': self.config['show_quick_navigation'],
                    'recent_items_limit': self.config['recent_items_limit'],
                    'items_per_section_limit': self.config['items_per_section_limit'],
                    'show_table_view': self.config['show_table_view'],
                    'table_sort_by': self.config['table_sort_by'],
                    'table_sort_order': self.config['table_sort_order'],
                    'max_table_rows': self.config['max_table_rows']
                }
            )
            
            # Parse dxlog project
            self.research_items = self.parser.parse_project()
            self.cross_references = self.parser.build_cross_references(self.research_items)
            
            logger.info(f"Loaded {len(self.research_items)} research items from dxlog")
            if not self.research_items:
                logger.warning(f"No research items found in {dxlog_dir}")
                logger.warning("Landing page will still be created but will be empty")
            
        except Exception as e:
            logger.error(f"Error initializing dxlog plugin: {e}")
        
        return config
    
    def on_files(self, files, config):
        """Add dxlog files to MkDocs file collection."""
        if not self.parser or not self.content_generator:
            return files
        
        try:
            # Generate virtual files for each research item
            for item_id, item in self.research_items.items():
                # Create file path for this research item
                file_path = self.content_generator.get_file_path(item)
                
                # Create virtual file
                virtual_file = File(
                    path=file_path,
                    src_dir=config['docs_dir'],
                    dest_dir=config['site_dir'],
                    use_directory_urls=config['use_directory_urls']
                )
                
                # Store item data for later use
                virtual_file.dxlog_item = item
                virtual_file.dxlog_id = item_id
                
                files.append(virtual_file)
            
            # Generate overview pages
            logger.info(f"Creating overview files for {len(self.research_items)} research items")
            overview_files = self.content_generator.create_overview_files(
                self.research_items, config
            )
            logger.info(f"Created {len(overview_files)} overview files")
            
            # Add each overview file individually (Files object doesn't have extend method)
            for overview_file in overview_files:
                logger.info(f"Adding overview file: {overview_file.src_path}")
                files.append(overview_file)
            
        except Exception as e:
            logger.error(f"Error adding dxlog files: {e}")
        
        return files
    
    def on_page_read_source(self, page: Page, config: Dict[str, Any]) -> Optional[str]:
        """Convert dxlog content to markdown."""
        if not hasattr(page.file, 'dxlog_item') and not hasattr(page.file, 'dxlog_overview') and not hasattr(page.file, 'dxlog_main_landing'):
            return None
        
        try:
            # Handle main landing page
            if hasattr(page.file, 'dxlog_main_landing'):
                logger.info(f"Generating content for main landing page: {page.file.src_path}")
                landing_data = page.file.dxlog_main_landing
                content = self.content_generator.generate_main_landing_content(landing_data)
                logger.info(f"Generated {len(content)} characters of landing page content")
                return content
            
            # Handle overview pages
            if hasattr(page.file, 'dxlog_overview'):
                overview_data = page.file.dxlog_overview
                content = self.content_generator.generate_overview_content(overview_data)
                return content
            
            # Handle individual item pages
            item = page.file.dxlog_item
            item_id = page.file.dxlog_id
            
            # Generate markdown content from dxlog item
            content = self.content_generator.generate_page_content(
                item, item_id, self.cross_references
            )
            
            return content
            
        except Exception as e:
            logger.error(f"Error converting dxlog content: {e}")
            return None
    
    def on_nav(self, nav: Navigation, config: Dict[str, Any], files) -> Navigation:
        """Modify navigation to include dxlog content."""
        if not self.content_generator:
            return nav
        
        try:
            # Generate navigation structure for dxlog content
            dxlog_nav = self.content_generator.generate_navigation(
                self.research_items, files
            )
            
            if dxlog_nav:
                # Add dxlog navigation to the main navigation
                nav.items.append(dxlog_nav)
            
        except Exception as e:
            logger.error(f"Error modifying navigation: {e}")
        
        return nav
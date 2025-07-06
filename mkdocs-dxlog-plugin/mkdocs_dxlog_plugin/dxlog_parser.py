"""Parser for dxlog research project files."""

import os
import re
import toml
import yaml
from typing import Dict, List, Any, Optional, Tuple
from pathlib import Path
import logging

logger = logging.getLogger(__name__)


class DxlogParser:
    """Parser for dxlog research project files."""
    
    def __init__(self, dxlog_dir: str):
        self.dxlog_dir = Path(dxlog_dir)
        self.config = self._load_config()
        
    def _load_config(self) -> Dict[str, Any]:
        """Load dxlog configuration from dxlog.toml."""
        config_path = self.dxlog_dir / "dxlog.toml"
        
        if not config_path.exists():
            logger.warning(f"No dxlog.toml found in {self.dxlog_dir}")
            return self._default_config()
        
        try:
            with open(config_path, 'r') as f:
                return toml.load(f)
        except Exception as e:
            logger.error(f"Error loading dxlog.toml: {e}")
            return self._default_config()
    
    def _default_config(self) -> Dict[str, Any]:
        """Return default dxlog configuration."""
        return {
            "storage": {
                "active-dir": "research-logs",
                "archive-dir": "archived", 
                "knowledge-base-dir": "knowledge-base"
            },
            "templates": {
                "hypothesis": "templates/hypothesis.jinja",
                "literature": "templates/literature.jinja",
                "knowledge": "templates/knowledge.jinja"
            }
        }
    
    def parse_project(self) -> Dict[str, Dict[str, Any]]:
        """Parse all research items in the dxlog project."""
        items = {}
        
        # Parse active research logs
        active_dir = self.dxlog_dir / self.config["storage"]["active-dir"]
        if active_dir.exists():
            items.update(self._parse_directory(active_dir, "active"))
        
        # Parse knowledge base
        kb_dir = self.dxlog_dir / self.config["storage"]["knowledge-base-dir"]
        if kb_dir.exists():
            items.update(self._parse_directory(kb_dir, "knowledge-base"))
        
        # Parse archived items
        archive_dir = self.dxlog_dir / self.config["storage"]["archive-dir"]
        if archive_dir.exists():
            items.update(self._parse_directory(archive_dir, "archived"))
        
        return items
    
    def _parse_directory(self, directory: Path, category: str) -> Dict[str, Dict[str, Any]]:
        """Parse all markdown files in a directory."""
        items = {}
        
        for md_file in directory.rglob("*.md"):
            try:
                item = self._parse_file(md_file)
                if item:
                    item["category"] = category
                    item["file_path"] = str(md_file.relative_to(self.dxlog_dir))
                    
                    # Use filename as ID if no UUID in frontmatter
                    item_id = item.get("id") or item.get("uuid") or md_file.stem
                    items[item_id] = item
                    
            except Exception as e:
                logger.error(f"Error parsing {md_file}: {e}")
        
        return items
    
    def _parse_file(self, file_path: Path) -> Optional[Dict[str, Any]]:
        """Parse a single markdown file with YAML frontmatter."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Split frontmatter and content
            frontmatter, markdown_content = self._split_frontmatter(content)
            
            if not frontmatter:
                logger.warning(f"No frontmatter found in {file_path}")
                return None
            
            # Parse YAML frontmatter
            try:
                metadata = yaml.safe_load(frontmatter)
                if not isinstance(metadata, dict):
                    metadata = {}
            except yaml.YAMLError as e:
                logger.error(f"Invalid YAML in {file_path}: {e}")
                return None
            
            # Determine item type from directory structure or frontmatter
            item_type = self._determine_item_type(file_path, metadata)
            
            return {
                "type": item_type,
                "title": metadata.get("title", file_path.stem),
                "content": markdown_content,
                "metadata": metadata,
                "file_path": str(file_path)
            }
            
        except Exception as e:
            logger.error(f"Error parsing file {file_path}: {e}")
            return None
    
    def _split_frontmatter(self, content: str) -> Tuple[str, str]:
        """Split YAML frontmatter from markdown content."""
        if not content.startswith("---"):
            return "", content
        
        # Find the end of frontmatter
        lines = content.split('\n')
        frontmatter_end = None
        
        for i, line in enumerate(lines[1:], 1):
            if line.strip() == "---":
                frontmatter_end = i
                break
        
        if frontmatter_end is None:
            return "", content
        
        frontmatter = '\n'.join(lines[1:frontmatter_end])
        markdown_content = '\n'.join(lines[frontmatter_end + 1:])
        
        return frontmatter, markdown_content
    
    def _determine_item_type(self, file_path: Path, metadata: Dict[str, Any]) -> str:
        """Determine the type of research item from file path or metadata."""
        # Check metadata first
        if "type" in metadata:
            return metadata["type"]
        
        # Determine from file path
        path_parts = file_path.parts
        
        if "hypothesis" in path_parts or "hypotheses" in path_parts:
            return "hypothesis"
        elif "literature" in path_parts:
            return "literature"
        elif "knowledge" in path_parts:
            return "knowledge"
        
        # Default to hypothesis
        return "hypothesis"
    
    def build_cross_references(self, items: Dict[str, Dict[str, Any]]) -> Dict[str, List[str]]:
        """Build cross-reference map between research items."""
        cross_refs = {}
        
        # Build UUID to item ID mapping
        uuid_to_id = {}
        for item_id, item in items.items():
            uuid = item.get("metadata", {}).get("uuid") or item.get("metadata", {}).get("id")
            if uuid:
                uuid_to_id[uuid] = item_id
        
        # Find references in content and frontmatter
        for item_id, item in items.items():
            refs = []
            
            # Check content for UUID references
            content_refs = self._find_references(item.get("content", ""), uuid_to_id)
            refs.extend(content_refs)
            
            # Check frontmatter references field
            frontmatter_refs = self._find_frontmatter_references(item.get("metadata", {}), uuid_to_id)
            refs.extend(frontmatter_refs)
            
            # Remove duplicates and add to cross_refs if any found
            if refs:
                cross_refs[item_id] = list(set(refs))
        
        return cross_refs
    
    def _find_references(self, content: str, uuid_to_id: Dict[str, str]) -> List[str]:
        """Find UUID references in markdown content."""
        references = []
        
        # Pattern to match UUID references (common UUID format)
        uuid_pattern = r'\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b'
        
        for match in re.finditer(uuid_pattern, content, re.IGNORECASE):
            uuid = match.group()
            if uuid in uuid_to_id:
                references.append(uuid_to_id[uuid])
        
        return references
    
    def _find_frontmatter_references(self, metadata: Dict[str, Any], uuid_to_id: Dict[str, str]) -> List[str]:
        """Find UUID references in frontmatter metadata."""
        references = []
        
        # Check the 'references' field in frontmatter
        refs_field = metadata.get("references", [])
        if isinstance(refs_field, list):
            for ref in refs_field:
                if isinstance(ref, str) and ref in uuid_to_id:
                    references.append(uuid_to_id[ref])
        elif isinstance(refs_field, str) and refs_field in uuid_to_id:
            references.append(uuid_to_id[refs_field])
        
        # Also check other potential reference fields
        for field_name in ["related", "depends_on", "references_to", "links_to"]:
            field_value = metadata.get(field_name, [])
            if isinstance(field_value, list):
                for ref in field_value:
                    if isinstance(ref, str) and ref in uuid_to_id:
                        references.append(uuid_to_id[ref])
            elif isinstance(field_value, str) and field_value in uuid_to_id:
                references.append(uuid_to_id[field_value])
        
        return references
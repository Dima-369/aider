from dataclasses import dataclass
from typing import Optional
import xml.etree.ElementTree as ET

@dataclass
class CommandRequest:
    command: str
    requires_approval: bool
    description: Optional[str] = None

def parse_command_xml(xml_content: str) -> CommandRequest:
    """Parse command XML and return structured command request"""
    root = ET.fromstring(xml_content)
    return CommandRequest(
        command=root.find('command').text.strip(),
        requires_approval=root.find('requires_approval').text.lower() == 'true',
        description=root.find('description').text if root.find('description') is not None else None
    )
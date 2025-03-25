from typing import Optional, Tuple
from ..run_cmd import run_cmd
from .base_coder import Coder

class ProcessCoder(Coder):
    def execute_command(self, command_request: 'CommandRequest') -> Tuple[int, str]:
        """
        Execute a command with proper safety checks and user approval
        Returns: Tuple of (exit_code, output)
        """
        if command_request.requires_approval:
            prompt = f"Run command: {command_request.command}"
            if command_request.description:
                prompt += f"\nDescription: {command_request.description}"
            
            if not self.io.confirm_ask(prompt, explicit_yes_required=True):
                return -1, "Command execution cancelled by user"

        return run_cmd(
            command_request.command,
            error_print=self.io.tool_error,
            cwd=self.root
        )

    def handle_command_xml(self, xml_content: str) -> Optional[str]:
        """Process command XML and return command output if successful"""
        try:
            command_request = parse_command_xml(xml_content)
            exit_code, output = self.execute_command(command_request)
            
            if exit_code == 0:
                if output.strip():
                    self.io.tool_output(output)
                return output
            else:
                self.io.tool_error(f"Command failed with exit code {exit_code}")
                self.io.tool_error(output)
                return None
                
        except Exception as e:
            self.io.tool_error(f"Error processing command: {e}")
            return None
import pytest
from aider.process_commands import CommandRequest, parse_command_xml
from aider.coders.process_coder import ProcessCoder
from unittest.mock import MagicMock, patch

def test_parse_command_xml():
    xml = """
    <execute_command>
        <command>ls -la</command>
        <requires_approval>true</requires_approval>
        <description>List directory contents</description>
    </execute_command>
    """
    request = parse_command_xml(xml)
    assert request.command == "ls -la"
    assert request.requires_approval is True
    assert request.description == "List directory contents"

def test_parse_command_xml_no_description():
    xml = """
    <execute_command>
        <command>pwd</command>
        <requires_approval>false</requires_approval>
    </execute_command>
    """
    request = parse_command_xml(xml)
    assert request.command == "pwd"
    assert request.requires_approval is False
    assert request.description is None

class TestProcessCoder:
    @pytest.fixture
    def mock_io(self):
        io = MagicMock()
        io.confirm_ask.return_value = True
        return io
    
    @pytest.fixture
    def process_coder(self, mock_io):
        coder = ProcessCoder(None)
        coder.io = mock_io
        return coder

    def test_execute_command_with_approval(self, process_coder):
        with patch('aider.run_cmd.run_cmd') as mock_run:
            mock_run.return_value = (0, "command output")
            request = CommandRequest("ls", requires_approval=True)
            
            exit_code, output = process_coder.execute_command(request)
            
            assert exit_code == 0
            assert output == "command output"
            process_coder.io.confirm_ask.assert_called_once()

    def test_execute_command_without_approval(self, process_coder):
        with patch('aider.run_cmd.run_cmd') as mock_run:
            mock_run.return_value = (0, "command output")
            request = CommandRequest("pwd", requires_approval=False)
            
            exit_code, output = process_coder.execute_command(request)
            
            assert exit_code == 0
            assert output == "command output"
            process_coder.io.confirm_ask.assert_not_called()
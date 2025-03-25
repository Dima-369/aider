# Available Commands

## File Operations
...

## Command Execution
You can execute system commands using the execute_command XML format. Always consider security implications and set requires_approval appropriately.

Example:
<execute_command>
<command>git status</command>
<requires_approval>false</requires_approval>
<description>Check git repository status</description>
</execute_command>

See the command execution guidelines for more details on when to require approval.
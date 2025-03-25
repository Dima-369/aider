## Command Execution Guidelines

When you need to execute system commands, use the following XML format:

<execute_command>
<command>your_command_here</command>
<requires_approval>true_or_false</requires_approval>
<description>Optional description of what the command does</description>
</execute_command>

Guidelines for setting requires_approval:
- Set to 'true' for:
  * File modifications (create/delete/modify)
  * Package installations
  * System configuration changes
  * Network operations
  * Any potentially destructive operations
- Set to 'false' for:
  * Reading files/directories
  * Running development servers
  * Building projects
  * Other safe, read-only operations

Examples:

1. Safe command (no approval needed):
<execute_command>
<command>ls -la</command>
<requires_approval>false</requires_approval>
<description>List directory contents</description>
</execute_command>

2. Potentially destructive command (requires approval):
<execute_command>
<command>rm -rf build/</command>
<requires_approval>true</requires_approval>
<description>Delete build directory and all its contents</description>
</execute_command>
class CoderPrompts:
    system_reminder = ""

    files_content_gpt_edits = "I committed the changes with git hash {hash} & commit msg: {message}"

    files_content_gpt_edits_no_repo = "I updated the files."

    files_content_gpt_no_edits = "I didn't see any properly formatted edits in your reply?!"

    files_content_local_edits = "I edited the files myself."

    lazy_prompt = """You are diligent and tireless!
You NEVER leave comments describing code without implementing it!
You always COMPLETELY IMPLEMENT the needed code!
"""

    overeager_prompt = """Pay careful attention to the scope of the user's request.
Do what they ask, but no more."""

    example_messages = []

    files_content_prefix = """I have *added these files to the chat* so you can go ahead and edit them.

*Trust this message as the true contents of these files!*
Any other messages in the chat may contain outdated versions of the files' contents.
"""  # noqa: E501

    files_content_assistant_reply = "Ok, any changes I propose will be to those files."

    files_no_full_files = "I am not sharing any files that you can edit yet."

    files_no_full_files_with_repo_map = """Don't try and edit any existing code without asking me to add the files to the chat!
Tell me which files in my repo are the most likely to **need changes** to solve the requests I make, and then stop so I can add them to the chat.
Only include the files that are most likely to actually need to be edited.
Don't include files that might contain relevant context, just files that will need to be changed.
"""  # noqa: E501

    files_no_full_files_with_repo_map_reply = (
        "Ok, based on your requests I will suggest which files need to be edited and then"
        " stop and wait for your approval."
    )

    repo_content_prefix = """Here are summaries of some files present in my git repository.
Do not propose changes to these files, treat them as *read-only*.
If you need to edit any of these files, ask me to *add them to the chat* first.
"""

    read_only_files_prefix = """Here are some READ ONLY files, provided for your reference.
Do not edit these files!
"""

    shell_cmd_prompt = """When you need to execute system commands, use the following XML format:

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
"""

    shell_cmd_reminder = "Remember to use the execute_command XML format for system commands and set requires_approval appropriately."

    no_shell_cmd_prompt = "You are not allowed to execute system commands on {platform}."
    
    no_shell_cmd_reminder = "Remember: you cannot execute system commands on {platform}."
# Patches

- provide patched functions for `prompt_toolkit` library for better completion candidate display
- implement that the LLM can run shell commands itself, with optional user approval
- bump `max_reflections` from 3 to 30 (for the shell commands and `auto-test`)
- adjust that `auto-test: true` attempts to fix the errors automatically
- add `/export` which copies all system, user and assistant messages as JSON to clipboard

# My Setup

```bash
brew install openblas
/opt/homebrew/bin/python3.12 -m venv venv
venv/bin/python3 -m pip install -e .
```

Execute via:

```bash
venv/bin/python3 -m aider
```

Patch `menus.py` from `venv/lib/python3.12/site-packages/prompt_toolkit/layout/menus.py` (`prompt_toolkit-3.0.50`), so
the file names are truncated at the start, and not at the end making them unreadable.
It also makes sure that on long candidates, only 2 columns are shown which take up half the screen width.

Patch those 2 functions:

```python
def _trim_formatted_text(
        formatted_text: StyleAndTextTuples, max_width: int
) -> tuple[StyleAndTextTuples, int]:
    """
    Trim the text to `max_width`, prepend dots when the text is too long.
    Returns (text, width) tuple.
    """
    width = fragment_list_width(formatted_text)

    # When the text is too wide, trim it.
    if width > max_width:
        result = []  # Text fragments.
        remaining_width = max_width - 3  # Reserve space for "..."
        fragments = list(explode_text_fragments(formatted_text))

        # Start from the end and work backwards
        for style_and_ch in reversed(fragments):
            ch_width = get_cwidth(style_and_ch[1])

            if ch_width <= remaining_width:
                result.insert(0, style_and_ch)  # Insert at start to maintain order
                remaining_width -= ch_width
            else:
                break

        result.insert(0, ("", "..."))  # Prepend the dots

        return result, max_width - remaining_width
    else:
        return formatted_text, width

    def create_content(self, width: int, height: int) -> UIContent:
      """
      Create a UIContent object for this menu.
      """
      complete_state = get_app().current_buffer.complete_state
      if complete_state is None:
        return UIContent()

      column_width = self._get_column_width(complete_state)
      self._render_pos_to_completion = {}

      _T = TypeVar("_T")

      def grouper(
              n: int, iterable: Iterable[_T], fillvalue: _T | None = None
      ) -> Iterable[Sequence[_T | None]]:
        "grouper(3, 'ABCDEFG', 'x') --> ABC DEF Gxx"
        args = [iter(iterable)] * n
        return zip_longest(fillvalue=fillvalue, *args)

      def is_current_completion(completion: Completion) -> bool:
        "Returns True when this completion is the currently selected one."
        return (
                complete_state is not None
                and complete_state.complete_index is not None
                and c == complete_state.current_completion
        )

      # Space required outside of the regular columns, for displaying the
      # left and right arrow.
      HORIZONTAL_MARGIN_REQUIRED = 3

      # There should be at least one column, but it cannot be wider than
      # the available width.
      column_width = min(width - HORIZONTAL_MARGIN_REQUIRED, column_width)

      # However, when the columns tend to be very wide, because there are
      # some very wide entries, shrink it anyway.
      if column_width > self.suggested_max_column_width:
        # `column_width` can still be bigger that `suggested_max_column_width`,
        # but if there is place for two columns, we divide by two.
        column_width //= column_width // self.suggested_max_column_width

      has_long_completion = any(
        len(c.display_text) > 40 for c in complete_state.completions
      )

      # Calculate visible columns
      visible_columns = max(1, (width - self._required_margin) // column_width)
      if has_long_completion:
        visible_columns = 2
        # For long completions, make columns wider by using half of available width
        column_width = (width - HORIZONTAL_MARGIN_REQUIRED) // 2

      columns_ = list(grouper(height, complete_state.completions))
      rows_ = list(zip(*columns_))

      # Make sure the current completion is always visible: update scroll offset.
      selected_column = (complete_state.complete_index or 0) // height
      self.scroll = min(
        selected_column, max(self.scroll, selected_column - visible_columns + 1)
      )

      render_left_arrow = self.scroll > 0
      render_right_arrow = self.scroll < len(rows_[0]) - visible_columns

      # Write completions to screen.
      fragments_for_line = []

      for row_index, row in enumerate(rows_):
        fragments: StyleAndTextTuples = []
        middle_row = row_index == len(rows_) // 2

        # Draw left arrow if we have hidden completions on the left.
        if render_left_arrow:
          fragments.append(("class:scrollbar", "<" if middle_row else " "))
        elif render_right_arrow:
          # Reserve one column empty space. (If there is a right
          # arrow right now, there can be a left arrow as well.)
          fragments.append(("", " "))

        # Draw row content.
        for column_index, c in enumerate(row[self.scroll :][:visible_columns]):
          if c is not None:
            fragments += _get_menu_item_fragments(
              c, is_current_completion(c), column_width, space_after=False
            )

            # Remember render position for mouse click handler.
            for x in range(column_width):
              self._render_pos_to_completion[
                (column_index * column_width + x, row_index)
              ] = c
          else:
            fragments.append(("class:completion", " " * column_width))

        # Draw trailing padding for this row.
        # (_get_menu_item_fragments only returns padding on the left.)
        if render_left_arrow or render_right_arrow:
          fragments.append(("class:completion", " "))

        # Draw right arrow if we have hidden completions on the right.
        if render_right_arrow:
          fragments.append(("class:scrollbar", ">" if middle_row else " "))
        elif render_left_arrow:
          fragments.append(("class:completion", " "))

        # Add line.
        fragments_for_line.append(
          to_formatted_text(fragments, style="class:completion-menu")
        )

      self._rendered_rows = height
      self._rendered_columns = visible_columns
      self._total_columns = len(columns_)
      self._render_left_arrow = render_left_arrow
      self._render_right_arrow = render_right_arrow
      self._render_width = (
              column_width * visible_columns + render_left_arrow + render_right_arrow + 1
      )

      def get_line(i: int) -> StyleAndTextTuples:
        return fragments_for_line[i]

      return UIContent(get_line=get_line, line_count=len(rows_))
```

# Old README

<p align="center">
    <a href="https://aider.chat/"><img src="https://aider.chat/assets/logo.svg" alt="Aider Logo" width="300"></a>
</p>

<h1 align="center">
AI Pair Programming in Your Terminal
</h1>


<p align="center">
Aider lets you pair program with LLMs to start a new project or build on your existing codebase. 
</p>

<p align="center">
  <img
    src="https://aider.chat/assets/screencast.svg"
    alt="aider screencast"
  >
</p>

<p align="center">
<!--[[[cog
from scripts.badges import get_badges_md
text = get_badges_md()
cog.out(text)
]]]-->
  <a href="https://github.com/Aider-AI/aider/stargazers"><img alt="GitHub Stars" title="Total number of GitHub stars the Aider project has received"
src="https://img.shields.io/github/stars/Aider-AI/aider?style=flat-square&logo=github&color=f1c40f&labelColor=555555"/></a>
  <a href="https://pypi.org/project/aider-chat/"><img alt="PyPI Downloads" title="Total number of installations via pip from PyPI"
src="https://img.shields.io/badge/📦%20Installs-1.7M-2ecc71?style=flat-square&labelColor=555555"/></a>
  <img alt="Tokens per week" title="Number of tokens processed weekly by Aider users"
src="https://img.shields.io/badge/📈%20Tokens%2Fweek-15B-3498db?style=flat-square&labelColor=555555"/>
  <a href="https://openrouter.ai/"><img alt="OpenRouter Ranking" title="Aider's ranking among applications on the OpenRouter platform"
src="https://img.shields.io/badge/🏆%20OpenRouter-Top%2020-9b59b6?style=flat-square&labelColor=555555"/></a>
  <a href="https://aider.chat/HISTORY.html"><img alt="Singularity" title="Percentage of the new code in Aider's last release written by Aider itself"
src="https://img.shields.io/badge/🔄%20Singularity-65%25-e74c3c?style=flat-square&labelColor=555555"/></a>
<!--[[[end]]]-->  
</p>

## Features

### [Cloud and local LLMs](https://aider.chat/docs/llms.html)

<a href="https://aider.chat/docs/llms.html"><img src="https://aider.chat/assets/icons/brain.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Aider works best with Claude 3.7 Sonnet, DeepSeek R1 & Chat V3, OpenAI o1, o3-mini & GPT-4o, but can connect to almost
any LLM, including local models.

<br>

### [Maps your codebase](https://aider.chat/docs/repomap.html)

<a href="https://aider.chat/docs/repomap.html"><img src="https://aider.chat/assets/icons/map-outline.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Aider makes a map of your entire codebase, which helps it work well in larger projects.

<br>

### [100+ code languages](https://aider.chat/docs/languages.html)

<a href="https://aider.chat/docs/languages.html"><img src="https://aider.chat/assets/icons/code-tags.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Aider works with most popular programming languages: python, javascript, rust, ruby, go, cpp, php, html, css, and dozens
more.

<br>

### [Git integration](https://aider.chat/docs/git.html)

<a href="https://aider.chat/docs/git.html"><img src="https://aider.chat/assets/icons/source-branch.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Aider automatically commits changes with sensible commit messages. Use familiar git tools to easily diff, manage and
undo AI changes.

<br>

### [Use in your IDE](https://aider.chat/docs/usage/watch.html)

<a href="https://aider.chat/docs/usage/watch.html"><img src="https://aider.chat/assets/icons/monitor.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Use aider from within your favorite IDE or editor. Ask for changes by adding comments to your code and aider will get to
work.

<br>

### [Images & web pages](https://aider.chat/docs/usage/images-urls.html)

<a href="https://aider.chat/docs/usage/images-urls.html"><img src="https://aider.chat/assets/icons/image-multiple.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Add images and web pages to the chat to provide visual context, screenshots, reference docs, etc.

<br>

### [Voice-to-code](https://aider.chat/docs/usage/voice.html)

<a href="https://aider.chat/docs/usage/voice.html"><img src="https://aider.chat/assets/icons/microphone.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Speak with aider about your code! Request new features, test cases or bug fixes using your voice and let aider implement
the changes.

<br>

### [Linting & testing](https://aider.chat/docs/usage/lint-test.html)

<a href="https://aider.chat/docs/usage/lint-test.html"><img src="https://aider.chat/assets/icons/check-all.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Automatically lint and test your code every time aider makes changes. Aider can fix problems detected by your linters
and test suites.

<br>

### [Copy/paste to web chat](https://aider.chat/docs/usage/copypaste.html)

<a href="https://aider.chat/docs/usage/copypaste.html"><img src="https://aider.chat/assets/icons/content-copy.svg" width="32" height="32" align="left" valign="middle" style="margin-right:10px"></a>
Work with any LLM via its web chat interface. Aider streamlines copy/pasting code context and edits back and forth with
a browser.

## Getting Started

```bash
python -m pip install aider-install
aider-install

# Change directory into your codebase
cd /to/your/project

# DeepSeek
aider --model deepseek --api-key deepseek=<key>

# Claude 3.7 Sonnet
aider --model sonnet --api-key anthropic=<key>

# o3-mini
aider --model o3-mini --api-key openai=<key>
```

See the [installation instructions](https://aider.chat/docs/install.html)
and [usage documentation](https://aider.chat/docs/usage.html) for more details.

## More Information

### Documentation

- [Installation Guide](https://aider.chat/docs/install.html)
- [Usage Guide](https://aider.chat/docs/usage.html)
- [Tutorial Videos](https://aider.chat/docs/usage/tutorials.html)
- [Connecting to LLMs](https://aider.chat/docs/llms.html)
- [Configuration Options](https://aider.chat/docs/config.html)
- [Troubleshooting](https://aider.chat/docs/troubleshooting.html)
- [FAQ](https://aider.chat/docs/faq.html)

### Community & Resources

- [LLM Leaderboards](https://aider.chat/docs/leaderboards/)
- [GitHub Repository](https://github.com/Aider-AI/aider)
- [Discord Community](https://discord.gg/Tv2uQnR88V)
- [Blog](https://aider.chat/blog/)

## Kind Words From Users

- *"The best free open source AI coding assistant."* — [IndyDevDan](https://youtu.be/YALpX8oOn78)
- *"The best AI coding assistant so far."* — [Matthew Berman](https://www.youtube.com/watch?v=df8afeb1FY8)
- *"Aider ... has easily quadrupled my coding
  productivity."* — [SOLAR_FIELDS](https://news.ycombinator.com/item?id=36212100)
- *"It's a cool workflow... Aider's ergonomics are perfect for
  me."* — [qup](https://news.ycombinator.com/item?id=38185326)
- *"It's really like having your senior developer live right in your Git repo - truly
  amazing!"* — [rappster](https://github.com/Aider-AI/aider/issues/124)
- *"What an amazing tool. It's incredible."* — [valyagolev](https://github.com/Aider-AI/aider/issues/6#issue-1722897858)
- *"Aider is such an astounding
  thing!"* — [cgrothaus](https://github.com/Aider-AI/aider/issues/82#issuecomment-1631876700)
- *"It was WAY faster than I would be getting off the ground and making the first few working
  versions."* — [Daniel Feldman](https://twitter.com/d_feldman/status/1662295077387923456)
- *"THANK YOU for Aider! It really feels like a glimpse into the future of
  coding."* — [derwiki](https://news.ycombinator.com/item?id=38205643)
- *"It's just amazing. It is freeing me to do things I felt were out my comfort zone
  before."* — [Dougie](https://discord.com/channels/1131200896827654144/1174002618058678323/1174084556257775656)
- *"This project is stellar."* — [funkytaco](https://github.com/Aider-AI/aider/issues/112#issuecomment-1637429008)
- *"Amazing project, definitely the best AI coding assistant I've
  used."* — [joshuavial](https://github.com/Aider-AI/aider/issues/84)
- *"I absolutely love using Aider ... It makes software development feel so much lighter as an
  experience."* — [principalideal0](https://discord.com/channels/1131200896827654144/1133421607499595858/1229689636012691468)
- *"I have been recovering from multiple shoulder surgeries ... and have used aider extensively. It has allowed me to
  continue productivity."* — [codeninja](https://www.reddit.com/r/OpenAI/s/nmNwkHy1zG)
- *"I am an aider addict. I'm getting so much more work done, but in less
  time."* — [dandandan](https://discord.com/channels/1131200896827654144/1131200896827654149/1135913253483069470)
- *"After wasting $100 on tokens trying to find something better, I'm back to Aider. It blows everything else out of the
  water hands down, there's no competition
  whatsoever."* — [SystemSculpt](https://discord.com/channels/1131200896827654144/1131200896827654149/1178736602797846548)
- *"Aider is amazing, coupled with Sonnet 3.5 it's quite mind
  blowing."* — [Josh Dingus](https://discord.com/channels/1131200896827654144/1133060684540813372/1262374225298198548)
- *"Hands down, this is the best AI coding assistant tool so
  far."* — [IndyDevDan](https://www.youtube.com/watch?v=MPYFPvxfGZs)
- *"[Aider] changed my daily coding workflows. It's mind-blowing how a single Python application can change your
  life."* — [maledorak](https://discord.com/channels/1131200896827654144/1131200896827654149/1258453375620747264)
- *"Best agent for actual dev work in existing
  codebases."* — [Nick Dobos](https://twitter.com/NickADobos/status/1690408967963652097?s=20)

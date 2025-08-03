Analysis and plan: Handling AI responses that reference files

Goal
Define how the AI can reference file names and how the system should detect and include the referenced file contents in the request bodies sent to the AI on subsequent turns.

Scope
- Parsing AI responses to detect file references
- Resolving file references to workspace paths
- Reading file contents safely and efficiently
- Assembling request bodies with inlined file contents
- Edge cases, validation, and security
- Testing strategy

High-level approach
1) Allow the AI to reference files concisely using recognizable patterns.
2) Parse the AI response to extract file paths using robust rules.
3) Validate and resolve file paths within the workspace.
4) Read and include file contents using size limits and safety checks.
5) Annotate the request body with clear markers so the AI knows which file is included and from where.
6) Avoid re-reading the same file multiple times if referenced repeatedly.
7) Provide feedback to the user/AI if a file is missing, too large, or disallowed.

Accepted file reference patterns
- Inline bare relative paths: aider/utils.py, tests/basic/test_io.py
- Code-fence path headers: ```path=aider/utils.py```, ```file: aider/utils.py```, ```aider/utils.py```
- Colon-prefixed paths in bullets: - file: aider/utils.py
- Block directives:
  include: aider/utils.py
  include: ./aider/utils.py#L10-L40    (line ranges)
- Diff-like headers (e.g., in patches):
  --- a/aider/utils.py
  +++ b/aider/utils.py
- GitHub-style snippets:
  aider/utils.py#L10-L40

Parsing rules
- A path-like token matches any of:
  - Relative path with / separators and a known file extension from repo
  - Tests and code files under known workspace directories
  - Optional line anchors: #Lstart-Lend
- Ignore URLs (http://, https://), anchors without a recognized path, and absolute paths outside the workspace.
- Support quoted forms: "aider/utils.py", 'aider/utils.py'.
- Support list formats in AI responses.

Path resolution
- Normalize to workspace root using os.path.normpath.
- Reject path traversal (any .. that resolves outside root).
- Enforce allowlist: only files that exist under repo root.
- Optionally support glob patterns if explicitly enabled, resolving to multiple files.

Inclusion behavior
- For each valid referenced file:
  - Read file content up to a configured size limit (eg 256 KB per file, 2 MB total).
  - If a line range is specified, slice those lines (1-indexed, inclusive).
  - Deduplicate repeated references in one message.
  - Record metadata: path, size, hash (eg sha256), and slice range if any.
- Construct a structured segment for each included file in the next request body, for example:
  <<FILE_START path="aider/utils.py" sha256="..." lines="10-40">>
  <content>
  <<FILE_END>>
- Alternatively, present as a code fence with a header:
  ```file: aider/utils.py lines=10-40 sha256=...
  <content>
  ```
- If the file exceeds limits, include a notice block instead of content:
  <<FILE_TOO_LARGE path="..." size=... limit=...>>

Does it iterate over every token/word? Parsing strategy
- Do NOT iterate over every word; instead use efficient pattern matching:
  - Run a small set of compiled regexes against the entire response text to find candidate paths.
  - Post-filter matches by checking existence in the workspace and extension allowlist.
  - Example regexes:
    - (?P<path>[\w\-./]+\/[\w\-./]+\.[\w]+)(?:#L(?P<start>\d+)(?:-L(?P<end>\d+))?)?
    - ^```(?:file:|path=)?\s*(?P<path>[^\n`]+)
    - ^\s*include:\s*(?P<path>[^\n]+)
  - Also parse diff headers with ^---\s+[ab]\/(.+)$ and ^\+\+\+\s+[ab]\/(.+)$

Security and safety
- Prevent path traversal; ensure realpath lies under root.
- Enforce maximum number of files per response (eg 20) and total bytes.
- Binary detection: skip non-text files unless a binary allowlist. Optionally include hex or base64 if explicitly requested and size-permits.
- Sanitize control characters.

User/AI feedback
- Summarize included files after parsing the AI response:
  Included files (3): aider/utils.py (3.2 KB), tests/basic/test_io.py (1.1 KB)
- For missing files, reply with:
  Missing: tests/does_not_exist.py
- For large files, reply with:
  Skipped (too large): big/data.json (4.2 MB > 256 KB limit)
- Provide a guidance blurb in system prompts encouraging AI to use explicit file paths and optional line ranges.

Caching and performance
- Cache file size and hash to avoid repeated disk reads across a conversation turn.
- If the same file is requested multiple turns with same line range, read from cache.

Python example: extract and include files

import os
import re
from pathlib import Path

ROOT = Path.cwd()
MAX_FILE_SIZE = 256 * 1024
MAX_TOTAL_BYTES = 2 * 1024 * 1024

PATH_RE = re.compile(r"(?P<path>[\w\-./]+\/[\w\-./]+\.[\w]+)(?:#L(?P<start>\d+)(?:-L(?P<end>\d+))?)?")
FENCE_RE = re.compile(r"^```(?:file:|path=)?\s*(?P<path>[^\n`]+)", re.MULTILINE)
INCLUDE_RE = re.compile(r"^\s*include:\s*(?P<path>[^\n]+)", re.MULTILINE)


def safe_resolve(path_str: str) -> Path | None:
    p = (ROOT / path_str).resolve()
    try:
        p.relative_to(ROOT)
    except ValueError:
        return None
    return p if p.exists() and p.is_file() else None


def parse_candidates(text: str) -> list[tuple[str, int | None, int | None]]:
    cands = []
    for rx in (FENCE_RE, INCLUDE_RE, PATH_RE):
        for m in rx.finditer(text):
            raw = m.group("path").strip().strip("'\"")
            # Optional line anchors in PATH_RE groups
            if "start" in m.groupdict():
                start = int(m.group("start")) if m.group("start") else None
                end = int(m.group("end")) if m.group("end") else None
            else:
                # Parse anchors if present after #L
                start = end = None
                if "#L" in raw:
                    try:
                        path_part, anchor = raw.split("#L", 1)
                        raw = path_part
                        if "-L" in anchor:
                            s, e = anchor.split("-L", 1)
                            start, end = int(s), int(e)
                        else:
                            start = int(anchor)
                    except Exception:
                        start = end = None
            cands.append((raw, start, end))
    # Deduplicate while preserving order
    seen = set()
    out = []
    for path, s, e in cands:
        key = (path, s, e)
        if key not in seen:
            seen.add(key)
            out.append((path, s, e))
    return out


def read_slice(p: Path, start: int | None, end: int | None) -> str:
    text = p.read_text(errors="replace")
    if start is None:
        return text
    lines = text.splitlines()
    s = max(1, start)
    e = min(len(lines), end if end is not None else start)
    return "\n".join(lines[s-1:e])


def assemble_includes(response_text: str):
    total = 0
    includes = []
    for raw, s, e in parse_candidates(response_text):
        p = safe_resolve(raw)
        if not p:
            includes.append({"path": raw, "status": "missing"})
            continue
        size = p.stat().st_size
        if size > MAX_FILE_SIZE:
            includes.append({"path": raw, "status": "too_large", "size": size})
            continue
        content = read_slice(p, s, e)
        total += len(content.encode())
        if total > MAX_TOTAL_BYTES:
            includes.append({"path": raw, "status": "skipped_total_limit"})
            continue
        includes.append({"path": raw, "content": content, "lines": (s, e)})
    return includes


# Example usage
# response_text = "Please read aider/utils.py#L10-L40 and tests/basic/test_io.py"
# files = assemble_includes(response_text)
# Then add each file block to the next request body.

Testing strategy
- Unit tests for parsing:
  - Each pattern form (bare path, fence, include:, diff headers, anchors)
  - Deduplication
  - Path traversal rejection
  - Missing files and large file handling
- Integration tests:
  - AI responses with multiple files and line ranges
  - Ensure included blocks appear in the next request body and are capped by limits
  - Validate total byte limits and count of files limits
- Property-based tests on random text with embedded paths to ensure no excessive false positives.

FAQ
- Does it go over every returned word in the AI response and test if it is a file path?
  No. It uses a small set of targeted regular expressions to find likely file references, then validates them against the workspace. This is faster and less error-prone than scanning each token individually.

- Can binary files be included?
  By default, no. Add an allowlist or base64 with explicit request.

- How are conflicts handled if the AI asks for both a whole file and a line range from the same file?
  Prefer the more specific range. If both are needed and limits permit, include both separately but deduplicate identical ranges.

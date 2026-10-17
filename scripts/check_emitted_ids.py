#!/usr/bin/env python3
"""Check that every rule id the crate emits is registered in its rule table.

`SbmlIssue::new` reads the kind and the severity of a rule from the crate's rule table and panics
when the id is not there. That panic is the intended behaviour (an unregistered id has no defined
severity), but it is a runtime failure on the path that reports it, which is exactly the path a
test corpus may never reach. This check closes that gap statically: it collects every rule id the
sources can pass to the issue constructor and verifies that the table describes it.

How an id is collected:

  * the id argument of a `SbmlIssue::new(..)` call, and the second argument of
    `SbmlIssue::new_in(..)` (whose first argument is the rule set to resolve against);
  * when that argument is a string literal, the literal itself;
  * when it is a name, every rule-id-shaped literal in the expression that name is bound to (a
    `let`, a `const` or a `static`), plus every rule-id-shaped literal in the body of any function that expression calls (the
    dynamic tables of the type check resolve a tag to an id that way), and, when the variable is a
    function parameter, the corresponding argument of every call to that function.

The check is one-directional (`emitted ⊆ registered`): a registered rule that nothing emits is
legitimate, an emitted rule that is not registered is not. It is deliberately loud: an argument it
cannot classify is a failure, never a warning, because a silent pass would hide the very panic this
check exists to prevent. Code gated by `#[cfg(test)]` and files under a `tests/` directory are
skipped, so a test may use an id that is not a rule of the crate (for example to test the panic).

Usage::

    python3 scripts/check_emitted_ids.py
    python3 scripts/check_emitted_ids.py --crate /path/to/biodivine-lib-sbml
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_rule_ids import MIN_RULES, read_table  # noqa: E402  (path set above)

# The issue constructors and the position of their rule id argument.
CONSTRUCTORS = {"new": 0, "new_in": 1}

# A rule id is either a specification rule id (`10301`, `fbc-21207`) or one of the crate's own
# lint ids (`unused-namespace`, `SANITY_CHECK`). An id-shaped literal is one that contains a digit,
# a hyphen or an underscore, or is entirely upper case; that separates every rule id from the XML
# tag and attribute names that the same tables resolve (`sbml`, `model`, `listOfSpecies`,
# `geneProductAssociation`, ...), which no rule id can be confused with.
QUOTED_TEXT = re.compile(r'"([^"]*)"')


def is_rule_id_like(value: str) -> bool:
    """True when a string literal can only be a rule id, not an XML name."""

    if not re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_:.-]*", value):
        return False
    return (
        any(character.isdigit() for character in value)
        or "-" in value
        or "_" in value
        or value.isupper()
    )

LITERAL = re.compile(r'^"([^"]*)"$')


def strip_comments(text: str) -> str:
    """Remove `//` and `/* */` comments, so a literal in a comment is not collected."""

    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    text = re.sub(r"//[^\n]*", " ", text)
    return text


def drop_test_code(text: str) -> str:
    """Remove items gated by `#[cfg(test)]` (a test may report an id that is not a rule)."""

    out = text
    while True:
        match = re.search(r"#\[cfg\(test\)\]", out)
        if match is None:
            return out
        start = match.end()
        brace = out.find("{", start)
        semicolon = out.find(";", start)
        if brace < 0 or (0 <= semicolon < brace):
            # A declaration without a body (`#[cfg(test)] mod test_suite;`).
            out = out[: match.start()] + out[semicolon + 1 :] if semicolon >= 0 else out[: match.start()]
            continue
        depth = 0
        end = brace
        for index in range(brace, len(out)):
            if out[index] == "{":
                depth += 1
            elif out[index] == "}":
                depth -= 1
                if depth == 0:
                    end = index + 1
                    break
        out = out[: match.start()] + out[end:]


def call_arguments(text: str, name: str) -> list[tuple[list[str], int]]:
    """Return `(arguments, offset of the first argument)` for every `name(..)` call."""

    calls = []
    for match in re.finditer(r"(?<![\w.])" + re.escape(name) + r"\s*\(", text):
        start = match.end() - 1
        depth = 0
        in_string = False
        for index in range(start, len(text)):
            character = text[index]
            if in_string:
                if character == '"' and text[index - 1] != "\\":
                    in_string = False
            elif character == '"':
                in_string = True
            elif character == "(":
                depth += 1
            elif character == ")":
                depth -= 1
                if depth == 0:
                    break
        calls.append((split_arguments(text[start + 1 : index]), start + 1))
    return calls


def split_arguments(text: str) -> list[str]:
    """Split a call argument list on top-level commas."""

    arguments, depth, current, in_string = [], 0, "", False
    for index, character in enumerate(text):
        if in_string:
            current += character
            if character == '"' and text[index - 1] != "\\":
                in_string = False
        elif character == '"':
            in_string = True
            current += character
        elif character in "([{":
            depth += 1
            current += character
        elif character in ")]}":
            depth -= 1
            current += character
        elif character == "," and depth == 0:
            arguments.append(current.strip())
            current = ""
        else:
            current += character
    if current.strip() or arguments:
        arguments.append(current.strip())
    return arguments


def line_of(text: str, offset: int) -> int:
    return text[:offset].count("\n") + 1


def function_bodies(text: str) -> dict[str, list[str]]:
    """The body of every function defined in `text`, by name."""

    bodies: dict[str, list[str]] = {}
    for match in re.finditer(r"\bfn\s+(\w+)\s*(?:<[^>]*>)?\s*\(", text):
        brace = text.find("{", match.end())
        if brace < 0:
            continue
        depth = 0
        for index in range(brace, len(text)):
            if text[index] == "{":
                depth += 1
            elif text[index] == "}":
                depth -= 1
                if depth == 0:
                    bodies.setdefault(match.group(1), []).append(text[brace : index + 1])
                    break
    return bodies


def function_parameters(text: str) -> dict[str, list[str]]:
    """The parameter names of every function defined in `text`, by name."""

    parameters: dict[str, list[str]] = {}
    for match in re.finditer(r"\bfn\s+(\w+)\s*(?:<[^>]*>)?\s*\(", text):
        start = match.end() - 1
        depth = 0
        for index in range(start, len(text)):
            if text[index] == "(":
                depth += 1
            elif text[index] == ")":
                depth -= 1
                if depth == 0:
                    break
        names = []
        for argument in split_arguments(text[start + 1 : index]):
            name = re.match(r"(?:mut\s+)?(_?\w+)", argument.strip())
            if name is not None:
                names.append(name.group(1))
        parameters[match.group(1)] = names
    return parameters


def literals_in_expression(expression: str, bodies: dict[str, list[str]], depth: int = 0) -> set[str]:
    """Every id-shaped literal of an expression, including the bodies it calls (bounded depth)."""

    found = {
        value for value in QUOTED_TEXT.findall(expression) if is_rule_id_like(value)
    }
    if depth >= 4:
        return found
    for call in re.finditer(r"(?<![\w.])(\w+)\s*\(", expression):
        for body in bodies.get(call.group(1), []):
            found |= literals_in_expression(body, bodies, depth + 1)
    return found


def resolve_argument(
    argument: str,
    text: str,
    function: str | None,
    bodies: dict[str, list[str]],
    parameters: dict[str, list[str]],
    crate: Path,
    depth: int = 0,
) -> tuple[set[str], list[str]]:
    """Resolve one rule id argument to the ids it can be.

    Returns the ids that were found and the problems (arguments that could not be classified).
    A positive answer with no ids is itself a problem: it means the argument is a name this check
    could not follow, and assuming it is registered would defeat the check.
    """

    literal = LITERAL.match(argument)
    if literal is not None:
        return {literal.group(1)}, []

    name = argument.strip()
    if not re.fullmatch(r"\w+", name):
        return set(), [f"cannot classify the expression `{argument}`"]

    found: set[str] = set()

    # The name may be bound in the same file: a local `let`, or the `const`/`static` that a rule id
    # is usually declared as.
    binding = r"\b(?:let|const|static)\s+" + re.escape(name) + r"\s*(?::[^=;]+)?=\s*"
    for match in re.finditer(binding, text):
        statement = text[match.end() : text.find(";", match.end())]
        found |= literals_in_expression(statement, bodies)

    # ... or it may be a parameter of the enclosing function, in which case every call of that
    # function contributes its argument.
    if function is not None and name in parameters.get(function, []):
        index = parameters[function].index(name)
        for path in sorted((crate / "src").rglob("*.rs")):
            if "tests" in path.parts:
                continue
            caller = drop_test_code(strip_comments(path.read_text(encoding="utf-8")))
            call_bodies = function_bodies(caller)
            call_parameters = function_parameters(caller)
            for arguments, offset in call_arguments(caller, function):
                if index >= len(arguments):
                    continue
                location = f"{path}:{line_of(caller, offset)}"
                if depth >= 4:
                    return found, [f"{location}: the parameter `{name}` nests too deeply to follow"]
                nested, problems = resolve_argument(
                    arguments[index],
                    caller,
                    enclosing_function(caller, offset, call_bodies),
                    call_bodies,
                    call_parameters,
                    crate,
                    depth + 1,
                )
                found |= nested
                problems.extend(problems if not problems else problems)
                if problems:
                    return found, problems

    if not found:
        return set(), [
            f"the rule id argument `{name}` does not resolve to any rule id; extend this script "
            "rather than assuming it is registered"
        ]
    return found, []


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--list",
        action="store_true",
        help="print every collected rule id (useful to audit what the check covers)",
    )
    parser.add_argument(
        "--crate",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="path to the biodivine-lib-sbml crate (default: the crate containing this script)",
    )
    args = parser.parse_args()

    table_path = args.crate / "src" / "validation" / "rules.rs"
    registered = {rule_id for _number, rule_id, _kind in read_table(table_path)}
    if len(registered) < MIN_RULES:
        print(f"FAIL: {len(registered)} rule ids were read from {table_path}; that is not the table.")
        return 1

    sources = {}
    for path in sorted((args.crate / "src").rglob("*.rs")):
        if "tests" in path.parts:
            continue
        sources[path] = drop_test_code(strip_comments(path.read_text(encoding="utf-8")))

    # `emitted` maps a rule id to one place that can emit it, for the failure message.
    emitted: dict[str, str] = {}
    problems: list[str] = []

    for path, text in sources.items():
        bodies = function_bodies(text)
        parameters = function_parameters(text)
        for constructor, index in CONSTRUCTORS.items():
            for arguments, offset in call_arguments(text, f"SbmlIssue::{constructor}"):
                location = f"{path}:{line_of(text, offset)}"
                if index >= len(arguments):
                    problems.append(f"{location}: the call has no rule id argument")
                    continue
                found, found_problems = resolve_argument(
                    arguments[index],
                    text,
                    enclosing_function(text, offset, bodies),
                    bodies,
                    parameters,
                    args.crate,
                )
                for rule_id in found:
                    emitted.setdefault(rule_id, location)
                problems.extend(f"{location}: {problem}" for problem in found_problems)

    missing = sorted(rule_id for rule_id in emitted if rule_id not in registered)
    if missing:
        print(f"FAIL: {len(missing)} emitted rule id(s) are not registered in {table_path}:")
        for rule_id in missing:
            print(f"      {rule_id}  (emitted at {emitted[rule_id]})")
        print("      Add a descriptor to the rule table (the issue constructor panics without one).")
        return 1
    if problems:
        print(f"FAIL: {len(problems)} rule id argument(s) could not be classified:")
        for problem in problems:
            print(f"      {problem}")
        print("      Extend this script rather than assuming the argument is registered.")
        return 1

    if args.list:
        for rule_id in sorted(emitted):
            print(f"{rule_id}\t{emitted[rule_id]}")
    print(
        f"Emitted rule ids: {len(emitted)} distinct ids collected from src/, all registered "
        f"in a table of {len(registered)} rules ({len(sources)} files scanned)."
    )
    return 0


def enclosing_function(text: str, offset: int, bodies: dict[str, list[str]]) -> str | None:
    """The name of the function whose body contains `offset`, if it can be determined."""

    best: tuple[int, str] | None = None
    for name, body_bodies in bodies.items():
        for body in body_bodies:
            start = text.find(body)
            if start >= 0 and start <= offset <= start + len(body):
                if best is None or start > best[0]:
                    best = (start, name)
    return best[1] if best is not None else None


if __name__ == "__main__":
    sys.exit(main())

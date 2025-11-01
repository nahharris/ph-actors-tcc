# Atomic Conventional Commits

Create atomic commits using the Conventional Commits specification. Each commit should represent a single logical change.

## Process

1. **Analyze Changes**: First, get the current git status with `git status --porcelain` to see all modified, added, and deleted files.

2. **Group Files by Logical Scope**: Group related files together based on:
   - Module/component (e.g., `src/api/`, `src/app/`, `src/terminal/`)
   - Functionality (e.g., all mock implementations, all core implementations)
   - Type of change (e.g., test fixes, mock updates, core logic)

3. **Determine Commit Type and Scope**:
   - **Type**: `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `perf`, `build`, `ci`, `chore`
   - **Scope**: Extract from the file path (e.g., `api`, `app`, `terminal`, `cache`, `ui`)
   - **Subject**: Imperative mood, lowercase, <50 chars, no period

4. **Create Atomic Commits**:
   - Stage related files together: `git add <file1> <file2> ...`
   - Create commit with appropriate message: `git commit -m "type(scope): subject"`
   - Repeat for each logical group

## Rules

- **One logical change per commit**: Each commit should be meaningful on its own
- **Follow conventional commits**: `type(scope): subject` format
- **Scope extraction**: Use the module name from the path (e.g., `src/app/ui.rs` → scope: `ui`)
- **Grouping strategy**:
  - Files in the same module doing the same type of work → single commit
  - Mock implementations across modules → separate commit per module (or one if it's a pattern change)
  - Core logic changes → separate from mock/test changes
  - Test updates → separate from implementation changes

## Examples

- Multiple files in same module with same change type:
  ```
  git add src/terminal/core.rs src/terminal/message.rs src/terminal/mock.rs
  git commit -m "refactor(terminal): migrate to mockall pattern"
  ```

- Test fixes in one module:
  ```
  git add src/shell/tests.rs
  git commit -m "test(shell): fix mock usage in tests"
  ```

- Related mock updates:
  ```
  git add src/api/lore/mock.rs src/app/cache/mailing_list/mock.rs src/app/cache/patch/mock.rs
  git commit -m "refactor(mock): update all mock implementations to use mockall"
  ```

## Execution

Analyze the current changes and create atomic commits. Show what will be committed before actually committing (dry-run style), then proceed with creating the commits.


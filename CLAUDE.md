# Claude Code Guidelines

## Diff Optimization
- **Minimal changes only**: Change only lines directly required for the task
- **Avoid unrelated modifications**: Don't alter existing code that works correctly
- **Optimize for review**: Every changed line adds cognitive load for reviewers

## Plans

All changes have plans which are present in the `plan` directory as markdown files with names `${RFC-3339 Time Stamp}-${Summary}.md`.
Always read the README.md file before making any changes, to ensure all changes are aligned with the overall project vision.

### Planning Approach

Plans should be resistant to being accidentally incorrect by focusing on concepts and requirements rather than brittle implementation details:

**Good**: Describe the behavior, structure, and relationships needed
- "Add QuickCheck dependency for property-based testing"
- "Test mathematical properties like order independence"

**Avoid**: Specific version numbers, exact code snippets, or implementation details that may change
- "Add quickcheck = '1.0'" (version may be outdated)
- "Add module with `mod property_tests {}`" (syntax may be wrong)
- "Use exactly this function signature: `fn test_...`" (may not compile)

Plans should capture the intent and requirements clearly enough that implementation can adapt to current reality while staying true to the goal.

## Commit Strategy

Commits should be bite-sized changes that keep the repository in a healthy state (all tests pass, formatting, etc).
Their goal is to allow reviewers to step through the phases of implementation when a feature requires many steps to be truly complete.
Being in a healthy state is important for historical inspection, especially via tools like git-bisect.

Commits should be small, but not so small that logically associated changes become separated from a reviewer looking at them independently.
For example, it's okay to include the addition of a new dependency along with the code that needs it for the first time.
This level of subdivision introduces more overhead than is useful, the reviewer can't see the motivation for the new dependency if it's alone.

Each commit should:
- Pass all existing tests
- Be self-contained and meaningful
- Have a clear, descriptive commit message
- Represent a logical step in the implementation process

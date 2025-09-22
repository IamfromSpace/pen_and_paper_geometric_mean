# Claude Code Guidelines

## Evolvability, Abstraction, and Testing

The primary property of a good code base is that it can evolve, because the most likely thing it will do is change.
This is usually just extension, but often a core dependency is outgrown, yielding sweeping changes.
We must optimize for change, or be bogged down.

Abstraction is the key mechanism to optimize for change.
The guiding idea here is that "all models are wrong some are useful."
We're looking for abstractions that aren't wrong for our particular domain, and as efficiently useful as possible.

When calculating the trajectory of a baseball, it suffices to know that gravity is 9.8m/s^2.
But when building a GPS satellite, this model is terribly wrong.
Amazingly, we need to account for Special and even General Relativity.
Those more robust models are _still_ wrong, as they don't unify with quantum physics, but they are wrong in a way that doesn't matter for the problem.

Our baseball _can_ use this model, but it won't benefit enough from being less wrong.
It will be overall worse, because the cognitive load is much higher.
Possibly we need to account for the variation of gravity across the poles or other factors; this isn't an excuse to solve things poorly.
But complexity needs to pay its keep.

Tests serve two purposes:
- Eliminate all doubts that the new code is correct
- Preserve the correctness of the code as future changes are introduced

Since the goal of old tests is to preserve old functionality, we must be _extremely_ cautious when modifying old tests.
That also implies that when writing new tests we must be _extremely_ careful to write tests in ways that won't force future implementers to change them.
It's not just wasteful but dangerous.

The key is, again, good abstraction.

## Diff Optimization
- **Minimal changes only**: Change only lines directly required for the task
- **Avoid unrelated modifications**: Don't alter existing code that works correctly
- **Optimize for review**: Every changed line adds cognitive load for reviewers
- **Sentences in Markdown Paragraphs**: Sentences within a paragraph should always be separated by newlines, so diffs highlight only sentences changed, not the entire paragraph.

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

# Coding Ground Rules

These rules apply to all code in this project. The project is primarily JavaScript/TypeScript, but the principles below apply across every language used here.

## Control flow syntax

- Always write `if () { ... }` and `for () { ... }` with a body block.
- Never write `if ();` or `for ();` — an empty statement after the condition is forbidden.
- Always use a block for control flow constructs (`if`, `else`, `for`, `while`, `do`), even when the body is a single statement or a one-line `return`.

```ts
// Good
if (user.isActive) {
    return user;
}

for (const item of items) {
    process(item);
}

// Bad
if (user.isActive) return user;

for (const item of items) process(item);

if (user.isActive);   // never
for (let i = 0; i < n; i++);   // never
```

## Ternaries

- Avoid all nested ternaries. If a condition needs more than one ternary to express, use `if`/`else` statements or extract a small function instead.

```ts
// Good
function describe(status) {
    if (status === "open") {
        return "Open";
    }

    if (status === "pending") {
        return "Pending";
    }

    return "Closed";
}

// Bad
const label = status === "open" ? "Open" : status === "pending" ? "Pending" : "Closed";
```

A single, simple ternary is fine when it improves readability.

## Paradigm

- Prefer a functional style where it fits: pure functions, immutability, mapping/filtering/reducing over data, avoiding hidden side effects.
- Do not force functional style when another paradigm fits better. Use whichever paradigm — functional, imperative, or object-oriented — produces the most maintainable and scalable solution for the problem at hand.
- The goal is maintainable, scalable code. Paradigm is a tool, not a rule.

## Naming

- Use complete, descriptive names. A reader should understand the name without context.
- Do not use cryptic abbreviations.
    - Write `error`, not `e`.
    - Write `event`, not `e`.
    - Write `button`, not `btn`.
    - Write `request`, not `req`. Write `response`, not `res`.
    - Write `index`, not `idx`. Write `element`, not `el`.
- Do not go to the opposite extreme. Names should be complete, not verbose for its own sake. `userAuthenticationServiceFactoryConfigurationOptions` is worse than `authConfig` when the surrounding code already makes the meaning clear.
- Well-known, conventional short names are acceptable in their normal scope: loop counters `i`, `j`, mathematical `x`, `y`, callback parameters that mirror standard signatures.

## Spacing and readability

- Add blank lines between logical groups of statements to make code easier to scan.
- Separate variable declarations, control flow blocks, and the final `return` from each other when it aids reading.
- Inside functions, group related statements together and put a blank line between groups.

```ts
// Good
function checkout(cart, user) {
    const items = cart.items;
    const total = sumPrices(items);

    if (total <= 0) {
        throw new Error("Cart is empty");
    }

    const order = createOrder(user, items, total);
    persist(order);

    return order;
}
```

Don't insert blank lines so aggressively that every other line is empty — the goal is readability, not visual padding.

## Branch naming

- All work branches follow the schema `fix/N-slug`, where `N` is an incrementing integer and `slug` is a short kebab-case description (3–5 words).
- Determine the next `N` by inspecting existing branches: `git branch -a --list 'origin/fix/*' 'fix/*'` and taking `max(N) + 1`. Numbers are global across the repo and never reused, even if a branch was deleted or its PR was closed without merging.
- One branch = one PR = one logical change. Don't pile unrelated work onto the same branch.
- This applies to every branch we create, not just iterate runs.

Examples:

```
fix/58-backup-delete-confirm-overflow
fix/59-document-branch-naming
fix/60-restart-policy-toggle
```

## Summary

Code in this project should be:

- Always block-bodied for control flow.
- Free of nested ternaries.
- Written in whichever paradigm best serves maintainability and scale, with a functional lean where natural.
- Named with complete, descriptive identifiers — never cryptic, never excessive.
- Spaced for readability.
- Placed on branches named `fix/N-slug` with a single logical change per branch.
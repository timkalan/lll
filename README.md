# lll - large language lint

Queries an LLM to get some lint messages that your other linters miss.
Not quite a code review, but an okay first pass.
Some diagnostics may be hallucinated.

## Usage

```bash
lll <file>
```

## Example

```
$ lll examples/drizzle.ts

[suggestion] The 'db' parameter shadows the imported 'db' variable
from the module scope. Consider renaming the parameter.
  | async function getUser(db: typeof db, id: number) {
  |   return db.select().from(users).where(eq(users.id, id));
  | }

[warning] The type 'Parameters<typeof getUser>[0]' refers to the
shadowed 'db' variable. This type might not be what is expected for
a Drizzle database instance.
  | async function deleteUser(db: Parameters<typeof getUser>[0], id: number) {
  |   return db.delete(users).where(eq(users.id, id));
  | }
```

## Install

```
cargo install lll
```

Requires a Gemini API key set as `GEMINI_API_KEY`.

# Assessment Rubric

Used for the four labs and the capstone (`practical-challenge.md`). Each
criterion is scored **Beginning / Developing / Proficient / Exceeds**.

**Mastery rule:** the capstone must reach **Proficient** in *Correctness & edge
cases* and in *Testing*; all other criteria must reach at least *Developing*.

---

## Criteria

### 1. Correctness & edge cases
| Level | Descriptor |
|---|---|
| Beginning | Happy path only; fails on empty input or malformed data. |
| Developing | Main cases correct; misses some edge cases (year-end, leap day, check digit 0). |
| Proficient | All required cases and the named edge cases handled; rejects malformed input cleanly. |
| Exceeds | Anticipates unstated edges (negative values, huge inputs, unicode) and documents them. |

### 2. Testing
| Level | Descriptor |
|---|---|
| Beginning | No tests, or tests only assert "no crash". |
| Developing | Happy-path tests exist; few negative cases. |
| Proficient | Positive **and** negative tests; at least one regression-style test per real risk. |
| Exceeds | Tests read as executable specifications; a test would catch the `0042 2026`-class bug. |

### 3. Decomposition & code quality
| Level | Descriptor |
|---|---|
| Beginning | One large function; cryptic names. |
| Developing | Some separation; inconsistent naming. |
| Proficient | Pure, focused functions; meaningful names; no duplication. |
| Exceeds | Clean module boundaries (domain vs. transport); a reader finds the rule fast. |

### 4. Domain reasoning
| Level | Descriptor |
|---|---|
| Developing | Implements the rule mechanically. |
| Proficient | Explains *why* the rule exists and names the tradeoff (false pos/neg, check digit 0, moving holidays). |
| Exceeds | Connects the choice to the legal/privacy context and to failure consequences. |

### 5. System integration *(Option B only)*
| Level | Descriptor |
|---|---|
| Developing | Endpoints exist and return data. |
| Proficient | Typed errors → correct HTTP codes (400/422/429/503); input validated. |
| Exceeds | Cross-cutting concerns considered (logging/request id, rate limiting, privacy of any stored data). |

### 6. Pragmatism under time pressure
| Level | Descriptor |
|---|---|
| Beginning | Ran out of time with nothing runnable. |
| Developing | Runnable core; polish missing. |
| Proficient | Working, tested core within the timebox; scope consciously trimmed. |
| Exceeds | Trimmed scope **and** documented what was deferred and why. |

---

## Scoring sheet

| Criterion | Beginning (1) | Developing (2) | Proficient (3) | Exceeds (4) | Notes |
|---|---|---|---|---|---|
| Correctness & edge cases | | | | | |
| Testing | | | | | |
| Decomposition & quality | | | | | |
| Domain reasoning | | | | | |
| System integration (Option B) | | | | | |
| Pragmatism | | | | | |

**Suggested weights:** Correctness 30%, Testing 20%, Decomposition 15%, Domain
reasoning 15%, Integration 10%, Pragmatism 10%.

---

## Feedback protocol

1. State one thing that is **correct and non-obvious** in the submission.
2. Name the **highest-value gap** and the smallest change that closes it.
3. If a criterion is below Developing, point to the exact session/lab to revisit
   (`00-design-inputs.md` maps criteria to sessions).

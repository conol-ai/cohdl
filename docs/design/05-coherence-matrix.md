# 5. Coherence Matrix

# Status

v2 — reset for the ground-up redesign, 2026-07-13. The v1 matrix scored bug-fixes and features against an existing implementation. There is no v2 implementation yet, so this matrix instead scores the redesign's own core mechanisms — the things note 2 and note 3 propose — against the same seven dimensions, so we catch blast-radius problems in the design itself before a single line of the new compiler is written.

# Purpose — unchanged

Treat every significant capability as a system disturbance, not an isolated addition. Dimensions: Concepts · Grammar · Oracle · Diagnostics · Netlist · Compat · Trust. Impact scale: Low · Med · High · Crit.

# Matrix — v2 core redesign mechanisms

| Candidate capability | Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|---|
| Wire dormant DRC rules (E003–E005, W003–W004) | Low | Low | **Crit** | **High** | Low | Med | **High** |
| Fix designator collision bug | Low | Low | Med | Med | **High** | **High** | **High** |
| MPN propagation to instances/BOM | Low | Low | Med | Low | **High** | Med | Med |
| Fix nested `fn` calls | Low | Low | Med | Med | **High** | Med | Med |
| Stable `cohdl check --json` verify API | Low | Low | Med | **Crit** | Low | **High** | Med |
| Formal error-code registry (public contract) | Low | Low | Med | **Crit** | Low | **Crit** | Med |
| Ship `cohdl fmt` (canonical form) | Low | **High** | Low | Low | Low | Med | Med |
| Intent annotations `#[intent(...)]` | **High** | Med | Low | Low | Low (none) | Med | **High** |
| Generate→check→repair loop (Layer 5) | Low | Low | **High** | **Crit** | Med | Med | **High** |
| RL environment (verdict = reward) | Med | Low | **Crit** | **High** | Low | Med | Med |
| Layout-constraint concept (the door) | **Crit** | **High** | **High** | Med | **High** | **High** | **High** |
| New codegen target (e.g. Altium/SPICE) | Low | Low | Med | Low | **High** | Med | Med |

(Compat reads N/A across the board because there is no released v2 source yet — nothing to break. This column becomes meaningful again the moment v2 ships its first stable release; from that point on, treat every future change exactly as v1's matrix did.)

# Reading the matrix — the two mechanisms with the widest blast radius

Pin connection-obligation typing is the mechanism that most directly rewrites v1's failure mode (silently-unconnected pins, dormant W003/W004-style checks) into a type-level guarantee — it scores High on Oracle/Diagnostics/Trust because it's doing the job a DRC rule used to do, earlier and more reliably. This is the mechanism to get right first; everything downstream (the narrowed DRC scope, the simplified BOM emitter) assumes it works.

Narrowed residual-DRC scope scores Crit on Oracle and High on Diagnostics/Trust because it's a subtraction — we are deliberately making DRC do less. The risk this row flags: if the type system doesn't actually cover everything DRC used to catch, narrowing DRC's scope without confirming the type system replaced it would silently reopen v1's exact failure mode (a check that used to exist quietly stops existing at all, instead of moving to a new layer). Mitigation, mandatory before narrowing DRC: for every v1 DRC rule (E001–E005, W001–W004), explicitly classify it as "becomes a type-system mechanism" or "stays as narrowed residual DRC," and don't delete/narrow anything until its replacement is designed. This classification itself should be a note-6 RFC before implementation starts.

Layout-constraint concept remains the single riskiest row, unchanged from v1 — still Crit/High across nearly every dimension, still gated behind a goal-change proposal (GC-002 territory), still not touched by this redesign.

# Coherence questions for any significant change — carried forward, one added

1. Which of the core concepts does this touch? Does it add a new one?
2. Which priority-ladder ranks does it strengthen or weaken?
3. Does it create a second way to do the same thing?
4. Does it introduce a **model exception** a user/AI must memorize?
5. Does it make the system **harder to explain**?
6. Does it change the diagnostic/error-code contract?
7. Does it keep the netlist a **faithful, lossless** projection of source?
8. Does it preserve reproducibility?
9. Does it preserve **designator stability**?
10. Does it stay inside the layout non-goal?
11. Does it scale with future complexity, or is it a local patch?
12. If we maintain it for five years, is it still worth the conceptual cost?
13. (New) If this narrows or removes a DRC check, has its type-system replacement been designed and verified to cover the same ground — or are we just deleting a safety net?

# Feature evaluation formula — unchanged

Worked example for v2: units-as-types → very high severity (an entire class of v1-unaddressed mistakes), high strategic alignment (gradeability and generatability — the model can no longer silently emit a wrong-unit spec, and the error appears at the exact line), moderate conceptual cost (a handful of new primitive types, learnable in one sitting) → very high value, build first.

# Output of using this note — unchanged

Every RFC must include a filled row of this matrix and answer any coherence question it scores High/Crit on.

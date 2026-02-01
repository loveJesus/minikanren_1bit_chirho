/-
  For God so loved the world - John 3:16 ☧
  Lakefile for miniKanren Formalization
-/

import Lake
open Lake DSL

package MiniKanrenChirho where
  -- Package configuration

require mathlib from git
  "https://github.com/leanprover-community/mathlib4" @ "master"

@[default_target]
lean_lib DomainChirho where
  srcDir := "."

lean_lib MiniKanrenChirhoLib where
  srcDir := "."
  roots := #[`MiniKanrenChirho]

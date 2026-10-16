**Taking #111 (plan item L1.3 of #106, decision D1), slice-wise.**

Slice 1 is open as a draft PR: `model/src/Registry.lex.tex` (language-1.1
semantic module, compression's Registry pattern) projects `model/ids.toml`
byte-identically through the new opt-in `cargo xtask validate-registry`; the
locked compiler accepts the module, and no existing gate, register branch, or
count moves. The edit-ban, the `@generated` header, and the RP-06/RP-07
projection switch are slice 2, which proceeds once the open register work
(#146, #82, #150, #151, #153, #155, #156) has rebased, under the §31 ordering
protocol. #112 and #113 unblock at slice 2, not before.

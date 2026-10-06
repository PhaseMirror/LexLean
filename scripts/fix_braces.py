import sys
content = open("packages/compression-main/crates/uorc-core/src/evaluator.rs").read()
# Let's count braces for impl StepMachine
# Wait, I can just use a simple regex replacing the end of `evaluate` properly if it's broken,
# or simply remove the extra brace.

lines = content.split('\n')
# The extra brace is at line 213, let's just print the end of impl StepMachine.

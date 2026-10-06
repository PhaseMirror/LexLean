import sys

content = open("packages/compression-main/crates/uorc-core/src/evaluator.rs").read()
# it appended the test at the very end of the file, outside of the mod tests block.
# Let's remove it from the end and put it inside `mod tests { ... }`

parts = content.split("    #[test]\n    fn test_store_operations()")
if len(parts) == 2:
    main_part = parts[0]
    test_part = "    #[test]\n    fn test_store_operations()" + parts[1]
    
    # find the last closing brace in main_part
    last_brace = main_part.rfind("}")
    
    new_content = main_part[:last_brace] + test_part + "\n}\n"
    with open("packages/compression-main/crates/uorc-core/src/evaluator.rs", "w") as f:
        f.write(new_content)


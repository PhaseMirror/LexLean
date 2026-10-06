import sys
content = open("packages/compression-main/crates/uorc-core/src/encoder.rs").read()

content = content.replace('let target = b"hellohello".to_vec();', 'let target = b"hellohellohellohello".to_vec();')
content = content.replace("""        let better_graph = GraphBody {
            instructions: vec![
                Instruction::Literal(b"hello".to_vec()), // 0
                Instruction::Reference(0),               // 1
                Instruction::Concat(0, 1),               // 2 -> "hellohello"
            ],
        };""", """        let better_graph = GraphBody {
            instructions: vec![
                Instruction::Literal(b"hello".to_vec()), // 0
                Instruction::Reference(0),               // 1
                Instruction::Concat(0, 1),               // 2 -> "hellohello"
                Instruction::Reference(2),               // 3 -> "hellohello"
                Instruction::Concat(2, 3),               // 4 -> "hellohellohellohello"
            ],
        };""")

content = content.replace('Instruction::Literal(b"hellohello".to_vec())', 'Instruction::Literal(b"hellohellohellohello".to_vec())')

with open("packages/compression-main/crates/uorc-core/src/encoder.rs", "w") as f:
    f.write(content)

module
public import Init
public import Compiler.TargetSyntax
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace Compiler.TargetOracle

@[expose] public def encodeNats : (items : List (Nat)) -> List (Compiler.TargetSyntax.Value)
  | List.nil => ([] : List (Compiler.TargetSyntax.Value))
  | List.cons item rest => (Compiler.TargetSyntax.Value.nat (item) :: encodeNats (rest))

@[expose] public def encodeInts : (items : List (Int)) -> List (Compiler.TargetSyntax.Value)
  | List.nil => ([] : List (Compiler.TargetSyntax.Value))
  | List.cons item rest => (Compiler.TargetSyntax.Value.int (item) :: encodeInts (rest))

@[expose] public def encodeI8s : (items : List (Int8)) -> List (Compiler.TargetSyntax.Value)
  | List.nil => ([] : List (Compiler.TargetSyntax.Value))
  | List.cons item rest => (Compiler.TargetSyntax.Value.i8 (item) :: encodeI8s (rest))

@[expose] public def encodeBools : (items : List (Bool)) -> List (Compiler.TargetSyntax.Value)
  | List.nil => ([] : List (Compiler.TargetSyntax.Value))
  | List.cons item rest => (Compiler.TargetSyntax.Value.bool (item) :: encodeBools (rest))

@[expose] public def encodeStrings : (items : List (String)) -> List (Compiler.TargetSyntax.Value)
  | List.nil => ([] : List (Compiler.TargetSyntax.Value))
  | List.cons item rest => (Compiler.TargetSyntax.Value.string (item) :: encodeStrings (rest))

@[expose] public def encodeNatStrings : (items : List ((Prod (Nat) (String)))) -> List (Compiler.TargetSyntax.Value)
  | List.nil => ([] : List (Compiler.TargetSyntax.Value))
  | List.cons item rest => (Compiler.TargetSyntax.Value.pair (Compiler.TargetSyntax.Value.nat ((item).1)) (Compiler.TargetSyntax.Value.string ((item).2)) :: encodeNatStrings (rest))

@[expose] public def encodeAdjacency : (items : List ((Prod (Nat) (List (Nat))))) -> List (Compiler.TargetSyntax.Value)
  | List.nil => ([] : List (Compiler.TargetSyntax.Value))
  | List.cons item rest => (Compiler.TargetSyntax.Value.pair (Compiler.TargetSyntax.Value.nat ((item).1)) (Compiler.TargetSyntax.Value.list (encodeNats ((item).2))) :: encodeAdjacency (rest))

end Compiler.TargetOracle

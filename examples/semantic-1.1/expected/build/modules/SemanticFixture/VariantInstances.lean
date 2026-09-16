module
public import Init
public import SemanticFixture.VariantTypes
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace SemanticFixture.VariantInstances

public class UsesDefault (A : Type) where
  inner : SemanticFixture.VariantTypes.DefaultValue (A)

public instance (priority := 1000) natUsesDefault : UsesDefault (Nat) where
  inner := SemanticFixture.VariantTypes.natDefault

end SemanticFixture.VariantInstances

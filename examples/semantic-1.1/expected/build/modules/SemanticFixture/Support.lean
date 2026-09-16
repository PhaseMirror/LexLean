module
public import Init
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace SemanticFixture.Support

public inductive RemoteFlag where
  | disabled
  | enabled

@[expose] public def remoteEnabled : Bool := true

end SemanticFixture.Support

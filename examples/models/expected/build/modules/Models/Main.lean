module
public import Init
public import Models.Glyphs
public import Models.Pipeline
public import Models.Recognizer
public import Models.Session
public import Models.Triage
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace Models.Main

@[expose] public def classify (glyph : Models.Glyphs.Glyph) : Nat := Models.Recognizer.DigitModel (glyph)

@[expose] public def classifyChecked (glyph : Models.Glyphs.Glyph) : Except ((Prod Bool Bool)) (Nat) := (let __checked1_input : Models.Glyphs.Glyph := glyph; (let __checked1_output : Nat := Models.Recognizer.RawDigitModel (__checked1_input); (if Models.Recognizer.recognizesCheck (__checked1_input) (__checked1_output) then Except.ok (__checked1_output) else Except.error (((true, false) : Prod Bool Bool)))))

@[expose] public def respond (presentation : Models.Triage.Presentation) : Except ((Prod Bool Bool)) (Nat) := (let __checked1_input : Models.Triage.Presentation := presentation; (if Models.Triage.symptomaticCheck (__checked1_input) then (let __checked1_output : Nat := Models.Pipeline.PipelineModel (__checked1_input); Except.ok (__checked1_output)) else Except.error (((false, false) : Prod Bool Bool))))

@[expose] public def admitCosts (costs : List (Nat)) : Except ((Prod Bool Bool)) (Except ((Prod Bool Bool)) (List (Nat))) := (let __checked1_input : List (Nat) := costs; (if Models.Pipeline.shortStreamCheck (__checked1_input) then (let __checked1_output : Except ((Prod Bool Bool)) (List (Nat)) := Models.Pipeline.StreamModel (__checked1_input); Except.ok (__checked1_output)) else Except.error (((false, false) : Prod Bool Bool))))

@[expose] public def step (window : Models.Session.Window) (cost : Nat) : Except ((Prod Bool Bool)) ((Prod (Models.Session.Window) (Nat))) := (let __checked1_state : Models.Session.Window := window; (let __checked1_input : Nat := cost; (if Models.Session.boundedCheck (__checked1_state) then (if Models.Session.affordableCheck (__checked1_state) (__checked1_input) then (let __checked1_step : (Prod (Models.Session.Window) (Nat)) := Models.Session.SessionModel (__checked1_state) (__checked1_input); Except.ok (__checked1_step)) else Except.error (((false, false) : Prod Bool Bool))) else Except.error (((false, true) : Prod Bool Bool)))))

public theorem classify_eight : (classify (Models.Glyphs.Glyph.g8) = 8) := by
  decide

@[expose] public def admission (outcome : Except ((Prod Bool Bool)) (Except ((Prod Bool Bool)) (List (Nat)))) : Nat := (match outcome with | Except.error _ => 0 | Except.ok scanned => (match scanned with | Except.error _ => 1 | Except.ok _ => 2))

public theorem admit_rejects_costly : (admission (admitCosts ((3 :: (40 :: ([] : List (Nat)))))) = 1) := by
  decide

public theorem admit_accepts_cheap : (admission (admitCosts ((3 :: (16 :: (9 :: ([] : List (Nat))))))) = 2) := by
  decide

end Models.Main

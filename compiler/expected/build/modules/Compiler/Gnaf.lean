module
public import Init
public import Compiler.TargetSemantics
public import Compiler.TargetSyntax
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace Compiler.Gnaf

namespace LexLeanRuntime

public class ToMathInt (α : Type) where
  toInt : α -> Int

public class Fixed (α : Type) extends ToMathInt α where
  fromInt : Int -> α
  minimum : Int
  maximum : Int
  bitAnd : α -> α -> α
  bitOr : α -> α -> α
  bitXor : α -> α -> α
  bitNot : α -> α
  shiftLeft : α -> UInt32 -> Option α
  shiftRight : α -> UInt32 -> Option α

public instance : ToMathInt Int where toInt := fun value => value

public instance : Fixed Int8 where
  toInt := Int8.toInt
  fromInt := Int8.ofInt
  minimum := -128
  maximum := 127
  bitAnd := Int8.land
  bitOr := Int8.lor
  bitXor := Int8.xor
  bitNot := Int8.complement
  shiftLeft := fun value amount => if amount.toNat < 8 then some (Int8.shiftLeft value (Int8.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 8 then some (Int8.shiftRight value (Int8.ofNat amount.toNat)) else none

public instance : Fixed Int16 where
  toInt := Int16.toInt
  fromInt := Int16.ofInt
  minimum := -32768
  maximum := 32767
  bitAnd := Int16.land
  bitOr := Int16.lor
  bitXor := Int16.xor
  bitNot := Int16.complement
  shiftLeft := fun value amount => if amount.toNat < 16 then some (Int16.shiftLeft value (Int16.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 16 then some (Int16.shiftRight value (Int16.ofNat amount.toNat)) else none

public instance : Fixed Int32 where
  toInt := Int32.toInt
  fromInt := Int32.ofInt
  minimum := -2147483648
  maximum := 2147483647
  bitAnd := Int32.land
  bitOr := Int32.lor
  bitXor := Int32.xor
  bitNot := Int32.complement
  shiftLeft := fun value amount => if amount.toNat < 32 then some (Int32.shiftLeft value (Int32.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 32 then some (Int32.shiftRight value (Int32.ofNat amount.toNat)) else none

public instance : Fixed Int64 where
  toInt := Int64.toInt
  fromInt := Int64.ofInt
  minimum := -9223372036854775808
  maximum := 9223372036854775807
  bitAnd := Int64.land
  bitOr := Int64.lor
  bitXor := Int64.xor
  bitNot := Int64.complement
  shiftLeft := fun value amount => if amount.toNat < 64 then some (Int64.shiftLeft value (Int64.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 64 then some (Int64.shiftRight value (Int64.ofNat amount.toNat)) else none

public instance : Fixed UInt8 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt8.ofInt
  minimum := 0
  maximum := 255
  bitAnd := UInt8.land
  bitOr := UInt8.lor
  bitXor := UInt8.xor
  bitNot := UInt8.complement
  shiftLeft := fun value amount => if amount.toNat < 8 then some (UInt8.shiftLeft value (UInt8.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 8 then some (UInt8.shiftRight value (UInt8.ofNat amount.toNat)) else none

public instance : Fixed UInt16 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt16.ofInt
  minimum := 0
  maximum := 65535
  bitAnd := UInt16.land
  bitOr := UInt16.lor
  bitXor := UInt16.xor
  bitNot := UInt16.complement
  shiftLeft := fun value amount => if amount.toNat < 16 then some (UInt16.shiftLeft value (UInt16.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 16 then some (UInt16.shiftRight value (UInt16.ofNat amount.toNat)) else none

public instance : Fixed UInt32 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt32.ofInt
  minimum := 0
  maximum := 4294967295
  bitAnd := UInt32.land
  bitOr := UInt32.lor
  bitXor := UInt32.xor
  bitNot := UInt32.complement
  shiftLeft := fun value amount => if amount.toNat < 32 then some (UInt32.shiftLeft value amount) else none
  shiftRight := fun value amount => if amount.toNat < 32 then some (UInt32.shiftRight value amount) else none

public instance : Fixed UInt64 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt64.ofInt
  minimum := 0
  maximum := 18446744073709551615
  bitAnd := UInt64.land
  bitOr := UInt64.lor
  bitXor := UInt64.xor
  bitNot := UInt64.complement
  shiftLeft := fun value amount => if amount.toNat < 64 then some (UInt64.shiftLeft value (UInt64.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 64 then some (UInt64.shiftRight value (UInt64.ofNat amount.toNat)) else none

@[expose] public def checkedFromInt {α : Type} [Fixed α] (value : Int) : Option α :=
  if value < Fixed.minimum (α := α) then none else if Fixed.maximum (α := α) < value then none else some (Fixed.fromInt value)

@[expose] public def checkedConvert {α β : Type} [ToMathInt α] [Fixed β] (value : α) : Option β :=
  checkedFromInt (ToMathInt.toInt value)

@[expose] public def checkedAdd {α : Type} [Fixed α] (left right : α) : Option α :=
  checkedFromInt (ToMathInt.toInt left + ToMathInt.toInt right)

@[expose] public def checkedSubtract {α : Type} [Fixed α] (left right : α) : Option α :=
  checkedFromInt (ToMathInt.toInt left - ToMathInt.toInt right)

@[expose] public def checkedMultiply {α : Type} [Fixed α] (left right : α) : Option α :=
  checkedFromInt (ToMathInt.toInt left * ToMathInt.toInt right)

@[expose] public def checkedNegate {α : Type} [Fixed α] (value : α) : Option α :=
  checkedFromInt (-ToMathInt.toInt value)

@[expose] public def checkedQuotient {α : Type} [Fixed α] (left right : α) : Option α :=
  if ToMathInt.toInt right = 0 then none else checkedFromInt (Int.tdiv (ToMathInt.toInt left) (ToMathInt.toInt right))

@[expose] public def checkedAddInt64 (left right : Int64) : Option Int64 :=
  let value := left + right
  if (0 < right && value < left) || (right < 0 && left < value) then none else some value

@[expose] public def checkedSubtractInt64 (left right : Int64) : Option Int64 :=
  let value := left - right
  if (0 < right && left < value) || (right < 0 && value < left) then none else some value

@[expose] public def checkedNegateInt64 (value : Int64) : Option Int64 :=
  if value == (-9223372036854775808 : Int64) then none else some (-value)

@[expose] public def magnitudeInt64 (value : Int64) : UInt64 :=
  let bits := value.toUInt64
  if value < 0 then 0 - bits else bits

@[expose] public def signedMagnitudeInt64 (negative : Bool) (value : UInt64) : Int64 :=
  (if negative then 0 - value else value).toInt64

@[expose] public def divideMagnitudeInt64 : Nat -> UInt64 -> UInt64 -> UInt64 -> UInt64 -> UInt64
  | 0, _, _, _, quotient => quotient
  | Nat.succ fuel, source, divisor, remainder, quotient =>
      let high := 9223372036854775808 <= source
      let source := source + source
      let remainder := remainder + remainder + if high then 1 else 0
      let quotient := quotient + quotient
      if divisor <= remainder then
        divideMagnitudeInt64 fuel source divisor (remainder - divisor) (quotient + 1)
      else
        divideMagnitudeInt64 fuel source divisor remainder quotient

@[expose] public def checkedQuotientInt64 (left right : Int64) : Option Int64 :=
  if right == 0 then none
  else if left == (-9223372036854775808 : Int64) && right == (-1 : Int64) then none
  else
    let negative := (left < 0) != (right < 0)
    some (signedMagnitudeInt64 negative
      (divideMagnitudeInt64 64 (magnitudeInt64 left) (magnitudeInt64 right) 0 0))

@[expose] public def multiplyMagnitudeInt64 : Nat -> UInt64 -> UInt64 -> UInt64 -> Bool -> Option UInt64
  | 0, _, _, accumulator, _ => some accumulator
  | Nat.succ fuel, source, multiplicand, accumulator, negative =>
      let high := 9223372036854775808 <= source
      let limit := if negative then 9223372036854775808 else 9223372036854775807
      let halfLimit := if negative then 4611686018427387904 else 4611686018427387903
      if halfLimit < accumulator then none
      else
        let doubled := accumulator + accumulator
        if high then
          if limit < multiplicand || limit - multiplicand < doubled then none
          else multiplyMagnitudeInt64 fuel (source + source) multiplicand
            (doubled + multiplicand) negative
        else
          multiplyMagnitudeInt64 fuel (source + source) multiplicand doubled negative

@[expose] public def checkedMultiplyInt64 (left right : Int64) : Option Int64 :=
  let negative := (left < 0) != (right < 0)
  match multiplyMagnitudeInt64 64 (magnitudeInt64 right) (magnitudeInt64 left) 0 negative with
  | none => none
  | some value => some (signedMagnitudeInt64 negative value)

@[expose, noinline] public def subtract {α : Type} [Sub α] (left right : α) : α := left - right
@[expose, noinline] public def multiply {α : Type} [Mul α] (left right : α) : α := left * right
@[expose, noinline] public def negate {α : Type} [Neg α] (value : α) : α := -value

public class Quotient (α : Type) where
  quotient : α -> α -> α
  remainder : α -> α -> α
  isZero : α -> Bool

public instance : Quotient Nat where
  quotient := Nat.div
  remainder := Nat.mod
  isZero := fun value => value == 0

public instance : Quotient Int where
  quotient := Int.tdiv
  remainder := Int.tmod
  isZero := fun value => value == 0

@[expose, noinline] public def quotient {α : Type} [Quotient α] (left right zeroCase : α) : α :=
  if Quotient.isZero right then zeroCase else Quotient.quotient left right

@[expose, noinline] public def remainder {α : Type} [Quotient α] (left right zeroCase : α) : α :=
  if Quotient.isZero right then zeroCase else Quotient.remainder left right

@[expose] public def bitAnd {α : Type} [Fixed α] (left right : α) : α := Fixed.bitAnd left right
@[expose] public def bitOr {α : Type} [Fixed α] (left right : α) : α := Fixed.bitOr left right
@[expose] public def bitXor {α : Type} [Fixed α] (left right : α) : α := Fixed.bitXor left right
@[expose] public def bitNot {α : Type} [Fixed α] (value : α) : α := Fixed.bitNot value
@[expose] public def shiftLeft {α : Type} [Fixed α] (value : α) (amount : UInt32) : Option α := Fixed.shiftLeft value amount
@[expose] public def shiftRight {α : Type} [Fixed α] (value : α) (amount : UInt32) : Option α := Fixed.shiftRight value amount

public class Appendable (α : Type) where append : α -> α -> α
public instance {α : Type} : Appendable (List α) where append := List.append
public instance : Appendable ByteArray where append := ByteArray.append
@[expose] public def append {α : Type} [Appendable α] (left right : α) : α := Appendable.append left right

public class Lengthable (α : Type) where length : α -> Nat
public instance {α : Type} : Lengthable (List α) where length := List.length
public instance : Lengthable ByteArray where length := ByteArray.size
public instance : Lengthable String where length := String.length
@[expose] public def length {α : Type} [Lengthable α] (value : α) : Nat := Lengthable.length value

@[expose] public def listIndex {α : Type} : List α -> Nat -> Option α
  | [], _ => none
  | head :: _, 0 => some head
  | _ :: tail, index + 1 => listIndex tail index

public class Indexable (α β : Type) where index : α -> Nat -> Option β
public instance {α : Type} : Indexable (List α) α where index := listIndex
public instance : Indexable ByteArray UInt8 where index := fun value offset => value.data[offset]?
@[expose, noinline] public def index {α β : Type} [Indexable α β] (value : α) (offset : Nat) : Option β := Indexable.index value offset

public class Sliceable (α : Type) where slice : α -> Nat -> Nat -> Option α
public instance {α : Type} : Sliceable (List α) where
  slice := fun value start count => if start + count <= value.length then some ((value.drop start).take count) else none
public instance : Sliceable ByteArray where
  slice := fun value start count => if start + count <= value.size then some (value.extract start (start + count)) else none
@[expose, noinline] public def slice {α : Type} [Sliceable α] (value : α) (start count : Nat) : Option α := Sliceable.slice value start count

@[expose, noinline] public def utf8Encode (value : String) : ByteArray := value.toUTF8
@[expose, noinline] public def utf8Decode (value : ByteArray) : Option String := String.fromUTF8? value
@[expose, noinline] public def compareBytes (left right : ByteArray) : Ordering := compare left.toList right.toList
@[expose] public def equal {α : Type} [BEq α] (left right : α) : Bool := left == right

@[expose, noinline] public def splitExact (value delimiter : String) (maximum : UInt32) : Option (List String) :=
  let fields := value.splitOn delimiter
  if delimiter.isEmpty || maximum.toNat < fields.length then none else some fields

@[expose, noinline] public def join (values : List String) (delimiter : String) : String := delimiter.intercalate values

public class Decimal (α : Type) where
  parse : String -> Option α
  format : α -> String

public instance : Decimal Int where
  parse := fun value => match value.toInt? with | some parsed => if toString parsed = value then some parsed else none | none => none
  format := toString

public instance {α : Type} [Fixed α] [ToString α] : Decimal α where
  parse := fun value => match value.toInt? with | some parsed => if toString parsed = value then checkedFromInt parsed else none | none => none
  format := toString

@[expose, noinline] public def parseDecimal {α : Type} [Decimal α] (value : String) : Option α := Decimal.parse value
@[expose, noinline] public def formatDecimal {α : Type} [Decimal α] (value : α) : String := Decimal.format value

end LexLeanRuntime

public inductive ClaimClass where
  | exact
  | normalForm
  | canonical
  | representationMinimal
  | comparisonTheorem
  | profileDefinedComparison
  | inputTotal
  | globalOptimal
  | argminComplete
  | paretoOptimal
  | frontierComplete
  | pointwiseEnvelopeComplete
  | queryFamilyAnswerComplete
  | useCaseGlobalOptimal
  | workloadArgminComplete
  | workloadParetoOptimal
  | workloadFrontierComplete
  | familyOptimal
  | competitiveBound
  | competitiveOptimal
  | asymptoticBound
  | asymptoticOptimal
  | useCaseClassComplete
  | useCaseClassAnswerComplete
  | maintainedUseCaseClass
  | restrictedUniverseOptimal
  | revisionPreserved
  | bestKnown
  | measuredBestAmongTested
  | heuristicSelected
  | instanceOptimal (_ : Nat) (_ : Nat)

public inductive ActionKind where
  | observation
  | preprocessing
  | advice
  | retainedState
  | dispatch
  | fallback
  | communication
  | randomness
  | scheduling
  | execution

public inductive Charge where
  | steps
  | constant (_ : Nat)
  | free
  | undeclared

public structure Action where
  kind : ActionKind
  charge : Charge

public inductive Boundary where
  | complete
  | preparedState
  | preparedPlan

public structure Machine where
  fuel : Nat
  actions : List (Action)
  boundary : Boundary
  preparationCommon : Bool

public inductive Selector where
  | fixed (_ : Nat)
  | dispatch (_ : Nat) (_ : Nat) (_ : Nat)

public structure Grammar where
  argument : Compiler.TargetSyntax.Ty
  result : Compiler.TargetSyntax.Ty
  plans : List (Compiler.TargetSyntax.Function)
  thresholds : List (Nat)

public inductive Carrier where
  | grammar (_ : Grammar)
  | internalPlans
  | optimizerOutput
  | discovered (_ : List (Nat))
  | cached

public inductive Completeness where
  | grammarEquality
  | missing
  | citesUniverseId
  | citesOptimizer

public inductive Scope where
  | grammarUniverse
  | calculusPrograms
  | rustPrograms

public inductive Objective where
  | scalar
  | vector

public structure Request where
  reference : Compiler.TargetSyntax.Program
  domain : List (Compiler.TargetSyntax.Value)
  machine : Machine
  carrier : Carrier
  completeness : Completeness
  objective : Objective
  claim : ClaimClass
  scope : Scope

public inductive Rejection where
  | emptyDomain
  | internalPlanUniverse
  | optimizerDefinedUniverse
  | discoveredUniverse
  | cachedUniverse
  | missingCompleteness
  | selfReferentialCompleteness
  | optimizerCompleteness
  | duplicateAction (_ : ActionKind)
  | unaccountedAction (_ : ActionKind)
  | hiddenCost (_ : ActionKind)
  | unrealizableAction (_ : ActionKind)
  | uncommonPreparation
  | scalarClaimOverPartialOrder
  | vectorClaimOverTotalOrder
  | uncoveredScope
  | unsupportedClaim

public inductive Status where
  | admitted (_ : Nat) (_ : Nat)
  | inadmissible
  | unresolved

public inductive Answer where
  | rejected (_ : Rejection)
  | argmin (_ : List (Nat)) (_ : Nat)
  | frontier (_ : List (Nat))
  | infeasible
  | incomplete

@[expose] public def kindIndex (kind : ActionKind) : Nat := (match kind with | ActionKind.observation => 0 | ActionKind.preprocessing => 1 | ActionKind.advice => 2 | ActionKind.retainedState => 3 | ActionKind.dispatch => 4 | ActionKind.fallback => 5 | ActionKind.communication => 6 | ActionKind.randomness => 7 | ActionKind.scheduling => 8 | ActionKind.execution => 9)

@[expose] public def sameKind (left : ActionKind) (right : ActionKind) : Bool := (Nat.beq (kindIndex (left)) (kindIndex (right)))

@[expose] public def performed (kind : ActionKind) : Bool := (match kind with | ActionKind.observation => true | ActionKind.preprocessing => false | ActionKind.advice => false | ActionKind.retainedState => false | ActionKind.dispatch => true | ActionKind.fallback => true | ActionKind.communication => false | ActionKind.randomness => false | ActionKind.scheduling => false | ActionKind.execution => true)

@[expose] public def preparation (kind : ActionKind) : Bool := (match kind with | ActionKind.observation => false | ActionKind.preprocessing => true | ActionKind.advice => true | ActionKind.retainedState => true | ActionKind.dispatch => false | ActionKind.fallback => false | ActionKind.communication => false | ActionKind.randomness => false | ActionKind.scheduling => false | ActionKind.execution => false)

@[expose] public def findAction : (actions : List (Action)) -> (wanted : ActionKind) -> Option (Charge)
  | List.nil, _wanted => Option.none
  | List.cons action rest, wanted => (if sameKind ((action).kind) (wanted) then Option.some ((action).charge) else findAction (rest) (wanted))

@[expose] public def countKind : (actions : List (Action)) -> (wanted : ActionKind) -> Nat
  | List.nil, _wanted => 0
  | List.cons action rest, wanted => ((if sameKind ((action).kind) (wanted) then 1 else 0) + countKind (rest) (wanted))

@[expose] public def checkDistinct : (actions : List (Action)) -> (all : List (Action)) -> Option (Rejection)
  | List.nil, _all => Option.none
  | List.cons action rest, all => (if (Nat.blt (1) (countKind (all) ((action).kind))) then Option.some (Rejection.duplicateAction ((action).kind)) else checkDistinct (rest) (all))

@[expose] public def checkPerformed (machine : Machine) (wanted : ActionKind) : Option (Rejection) := (match findAction ((machine).actions) (wanted) with | Option.none => Option.some (Rejection.unaccountedAction (wanted)) | Option.some charge => (match charge with | Charge.steps => Option.none | Charge.constant _ => Option.some (Rejection.hiddenCost (wanted)) | Charge.free => Option.some (Rejection.hiddenCost (wanted)) | Charge.undeclared => Option.some (Rejection.hiddenCost (wanted))))

@[expose] public def checkDeclared (machine : Machine) (action : Action) : Option (Rejection) := (let kind : ActionKind := (action).kind; (if performed (kind) then Option.none else (if preparation (kind) then (match (action).charge with | Charge.steps => Option.some (Rejection.hiddenCost (kind)) | Charge.constant amount => (if (Nat.beq (amount) (0)) then Option.some (Rejection.hiddenCost (kind)) else Option.none) | Charge.free => (match (machine).boundary with | Boundary.complete => Option.some (Rejection.uncommonPreparation) | Boundary.preparedState => (if (machine).preparationCommon then Option.none else Option.some (Rejection.uncommonPreparation)) | Boundary.preparedPlan => (if (machine).preparationCommon then Option.none else Option.some (Rejection.uncommonPreparation))) | Charge.undeclared => Option.some (Rejection.hiddenCost (kind))) else Option.some (Rejection.unrealizableAction (kind)))))

@[expose] public def checkAllDeclared : (machine : Machine) -> (actions : List (Action)) -> Option (Rejection)
  | _machine, List.nil => Option.none
  | machine, List.cons action rest => (match checkDeclared (machine) (action) with | Option.none => checkAllDeclared (machine) (rest) | Option.some rejection => Option.some (rejection))

@[expose] public def preparationCharge : (actions : List (Action)) -> Nat
  | List.nil => 0
  | List.cons action rest => ((if preparation ((action).kind) then (match (action).charge with | Charge.steps => 0 | Charge.constant amount => amount | Charge.free => 0 | Charge.undeclared => 0) else 0) + preparationCharge (rest))

@[expose] public def checkClaim (request : Request) : Option (Rejection) := (match (request).claim with | ClaimClass.exact => Option.some (Rejection.unsupportedClaim) | ClaimClass.normalForm => Option.some (Rejection.unsupportedClaim) | ClaimClass.canonical => Option.some (Rejection.unsupportedClaim) | ClaimClass.representationMinimal => Option.some (Rejection.unsupportedClaim) | ClaimClass.comparisonTheorem => Option.some (Rejection.unsupportedClaim) | ClaimClass.profileDefinedComparison => Option.some (Rejection.unsupportedClaim) | ClaimClass.inputTotal => Option.some (Rejection.unsupportedClaim) | ClaimClass.globalOptimal => (match (request).objective with | Objective.scalar => Option.none | Objective.vector => Option.some (Rejection.scalarClaimOverPartialOrder)) | ClaimClass.argminComplete => (match (request).objective with | Objective.scalar => Option.none | Objective.vector => Option.some (Rejection.scalarClaimOverPartialOrder)) | ClaimClass.paretoOptimal => (match (request).objective with | Objective.scalar => Option.some (Rejection.vectorClaimOverTotalOrder) | Objective.vector => Option.none) | ClaimClass.frontierComplete => (match (request).objective with | Objective.scalar => Option.some (Rejection.vectorClaimOverTotalOrder) | Objective.vector => Option.none) | ClaimClass.pointwiseEnvelopeComplete => Option.some (Rejection.unsupportedClaim) | ClaimClass.queryFamilyAnswerComplete => Option.some (Rejection.unsupportedClaim) | ClaimClass.useCaseGlobalOptimal => Option.some (Rejection.unsupportedClaim) | ClaimClass.workloadArgminComplete => Option.some (Rejection.unsupportedClaim) | ClaimClass.workloadParetoOptimal => Option.some (Rejection.unsupportedClaim) | ClaimClass.workloadFrontierComplete => Option.some (Rejection.unsupportedClaim) | ClaimClass.familyOptimal => Option.some (Rejection.unsupportedClaim) | ClaimClass.competitiveBound => Option.some (Rejection.unsupportedClaim) | ClaimClass.competitiveOptimal => Option.some (Rejection.unsupportedClaim) | ClaimClass.asymptoticBound => Option.some (Rejection.unsupportedClaim) | ClaimClass.asymptoticOptimal => Option.some (Rejection.unsupportedClaim) | ClaimClass.useCaseClassComplete => Option.some (Rejection.unsupportedClaim) | ClaimClass.useCaseClassAnswerComplete => Option.some (Rejection.unsupportedClaim) | ClaimClass.maintainedUseCaseClass => Option.some (Rejection.unsupportedClaim) | ClaimClass.restrictedUniverseOptimal => (match (request).objective with | Objective.scalar => Option.none | Objective.vector => Option.some (Rejection.scalarClaimOverPartialOrder)) | ClaimClass.revisionPreserved => Option.some (Rejection.unsupportedClaim) | ClaimClass.bestKnown => Option.some (Rejection.unsupportedClaim) | ClaimClass.measuredBestAmongTested => Option.some (Rejection.unsupportedClaim) | ClaimClass.heuristicSelected => Option.some (Rejection.unsupportedClaim) | ClaimClass.instanceOptimal _ _ => Option.some (Rejection.unsupportedClaim))

@[expose] public def validate (request : Request) : Option (Rejection) := (match (match (request).domain with | List.nil => Option.some (Rejection.emptyDomain) | List.cons _ _ => Option.none) with | Option.none => (match (match (request).carrier with | Carrier.grammar _ => Option.none | Carrier.internalPlans => Option.some (Rejection.internalPlanUniverse) | Carrier.optimizerOutput => Option.some (Rejection.optimizerDefinedUniverse) | Carrier.discovered _ => Option.some (Rejection.discoveredUniverse) | Carrier.cached => Option.some (Rejection.cachedUniverse)) with | Option.none => (match (match (request).completeness with | Completeness.grammarEquality => Option.none | Completeness.missing => Option.some (Rejection.missingCompleteness) | Completeness.citesUniverseId => Option.some (Rejection.selfReferentialCompleteness) | Completeness.citesOptimizer => Option.some (Rejection.optimizerCompleteness)) with | Option.none => (match checkDistinct (((request).machine).actions) (((request).machine).actions) with | Option.none => (match checkPerformed ((request).machine) (ActionKind.observation) with | Option.none => (match checkPerformed ((request).machine) (ActionKind.dispatch) with | Option.none => (match checkPerformed ((request).machine) (ActionKind.fallback) with | Option.none => (match checkPerformed ((request).machine) (ActionKind.execution) with | Option.none => (match checkAllDeclared ((request).machine) (((request).machine).actions) with | Option.none => (match checkClaim (request) with | Option.none => (match (match (request).scope with | Scope.grammarUniverse => Option.none | Scope.calculusPrograms => Option.some (Rejection.uncoveredScope) | Scope.rustPrograms => Option.some (Rejection.uncoveredScope)) with | Option.none => Option.none | Option.some rejection18 => Option.some (rejection18)) | Option.some rejection17 => Option.some (rejection17)) | Option.some rejection16 => Option.some (rejection16)) | Option.some rejection15 => Option.some (rejection15)) | Option.some rejection14 => Option.some (rejection14)) | Option.some rejection13 => Option.some (rejection13)) | Option.some rejection12 => Option.some (rejection12)) | Option.some rejection11 => Option.some (rejection11)) | Option.some rejection10 => Option.some (rejection10)) | Option.some rejection9 => Option.some (rejection9)) | Option.some rejection8 => Option.some (rejection8))

@[expose] public def fixedSelectors : (count : Nat) -> (next : Nat) -> List (Selector)
  | Nat.zero, _next => ([] : List (Selector))
  | Nat.succ remaining, next => (Selector.fixed (next) :: fixedSelectors (remaining) ((next + 1)))

@[expose] public def pairsFrom : (small : Nat) -> (candidates : List (Nat)) -> List ((Prod (Nat) (Nat)))
  | _small, List.nil => ([] : List ((Prod (Nat) (Nat))))
  | small, List.cons large rest => (if (Nat.beq (small) (large)) then pairsFrom (small) (rest) else ((small, large) :: pairsFrom (small) (rest)))

@[expose] public def orderedPairs : (smalls : List (Nat)) -> (all : List (Nat)) -> List ((Prod (Nat) (Nat)))
  | List.nil, _all => ([] : List ((Prod (Nat) (Nat))))
  | List.cons small rest, all => (LexLeanRuntime.append (pairsFrom (small) (all)) (orderedPairs (rest) (all)) : List ((Prod (Nat) (Nat))))

@[expose] public def indices : (count : Nat) -> (next : Nat) -> List (Nat)
  | Nat.zero, _next => ([] : List (Nat))
  | Nat.succ remaining, next => (next :: indices (remaining) ((next + 1)))

@[expose] public def dispatchFor : (threshold : Nat) -> (pairs : List ((Prod (Nat) (Nat)))) -> List (Selector)
  | _threshold, List.nil => ([] : List (Selector))
  | threshold, List.cons chosen rest => (Selector.dispatch (threshold) ((chosen).1) ((chosen).2) :: dispatchFor (threshold) (rest))

@[expose] public def dispatchSelectors : (thresholds : List (Nat)) -> (pairs : List ((Prod (Nat) (Nat)))) -> List (Selector)
  | List.nil, _pairs => ([] : List (Selector))
  | List.cons threshold rest, pairs => (LexLeanRuntime.append (dispatchFor (threshold) (pairs)) (dispatchSelectors (rest) (pairs)) : List (Selector))

@[expose] public def expand (grammar : Grammar) : List (Selector) := (let count : Nat := (LexLeanRuntime.length ((grammar).plans) : Nat); (let all : List (Nat) := indices (count) (0); (LexLeanRuntime.append (fixedSelectors (count) (0)) (dispatchSelectors ((grammar).thresholds) (orderedPairs (all) (all))) : List (Selector))))

@[expose] public def entryBody (selector : Selector) : Compiler.TargetSyntax.Expr := (match selector with | Selector.fixed plan => Compiler.TargetSyntax.Expr.call ((plan + 1)) ((Compiler.TargetSyntax.Expr.var (0) :: ([] : List (Compiler.TargetSyntax.Expr)))) | Selector.dispatch threshold small large => Compiler.TargetSyntax.Expr.cond (Compiler.TargetSyntax.Expr.prim (Compiler.TargetSyntax.Prim.natLt) ((Compiler.TargetSyntax.Expr.prim (Compiler.TargetSyntax.Prim.length) ((Compiler.TargetSyntax.Expr.var (0) :: ([] : List (Compiler.TargetSyntax.Expr)))) :: (Compiler.TargetSyntax.Expr.value (Compiler.TargetSyntax.Ty.nat) (Compiler.TargetSyntax.Value.nat (threshold)) :: ([] : List (Compiler.TargetSyntax.Expr)))))) (Compiler.TargetSyntax.Expr.call ((small + 1)) ((Compiler.TargetSyntax.Expr.var (0) :: ([] : List (Compiler.TargetSyntax.Expr))))) (Compiler.TargetSyntax.Expr.call ((large + 1)) ((Compiler.TargetSyntax.Expr.var (0) :: ([] : List (Compiler.TargetSyntax.Expr))))))

@[expose] public def realize (grammar : Grammar) (selector : Selector) : Compiler.TargetSyntax.Program := ({ adts := ([] : List (Compiler.TargetSyntax.Adt)), functions := (({ parameters := (0 :: ([] : List (Nat))), types := ((grammar).argument :: ([] : List (Compiler.TargetSyntax.Ty))), result := (grammar).result, body := entryBody (selector) } : Compiler.TargetSyntax.Function) :: (grammar).plans) } : Compiler.TargetSyntax.Program)

mutual
@[expose] public def valueEq : (left : Compiler.TargetSyntax.Value) -> (right : Compiler.TargetSyntax.Value) -> Bool
  | Compiler.TargetSyntax.Value.unit, right => (match right with | Compiler.TargetSyntax.Value.unit => true | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.bool leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.nat leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat rightItem => (Nat.beq (leftItem) (rightItem)) | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.int leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int rightItem => Compiler.TargetSemantics.sameOrder (Compiler.TargetSemantics.orderInt (leftItem) (rightItem)) (Compiler.TargetSyntax.Order.same) | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.u8 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.u16 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.u32 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.u64 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.i8 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.i16 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.i32 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.i64 leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.string leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.bytes leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes rightItem => (LexLeanRuntime.equal (leftItem) (rightItem) : Bool) | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.ordering leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering rightItem => Compiler.TargetSemantics.sameOrder (leftItem) (rightItem) | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.none, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => true | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.some leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some rightItem => valueEq (leftItem) (rightItem) | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.ok leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok rightItem => valueEq (leftItem) (rightItem) | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.error leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error rightItem => valueEq (leftItem) (rightItem) | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.list leftItem, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list rightItem => valuesEq (leftItem) (rightItem) | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.pair leftFirst leftSecond, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair rightFirst rightSecond => (valueEq (leftFirst) (rightFirst) && valueEq (leftSecond) (rightSecond)) | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.adt leftTag leftFields, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt rightTag rightFields => ((Nat.beq (leftTag) (rightTag)) && valuesEq (leftFields) (rightFields)) | Compiler.TargetSyntax.Value.closure _ _ => false)
  | Compiler.TargetSyntax.Value.closure leftFunction leftCaptures, right => (match right with | Compiler.TargetSyntax.Value.unit => false | Compiler.TargetSyntax.Value.bool _ => false | Compiler.TargetSyntax.Value.nat _ => false | Compiler.TargetSyntax.Value.int _ => false | Compiler.TargetSyntax.Value.u8 _ => false | Compiler.TargetSyntax.Value.u16 _ => false | Compiler.TargetSyntax.Value.u32 _ => false | Compiler.TargetSyntax.Value.u64 _ => false | Compiler.TargetSyntax.Value.i8 _ => false | Compiler.TargetSyntax.Value.i16 _ => false | Compiler.TargetSyntax.Value.i32 _ => false | Compiler.TargetSyntax.Value.i64 _ => false | Compiler.TargetSyntax.Value.string _ => false | Compiler.TargetSyntax.Value.bytes _ => false | Compiler.TargetSyntax.Value.ordering _ => false | Compiler.TargetSyntax.Value.none => false | Compiler.TargetSyntax.Value.some _ => false | Compiler.TargetSyntax.Value.ok _ => false | Compiler.TargetSyntax.Value.error _ => false | Compiler.TargetSyntax.Value.list _ => false | Compiler.TargetSyntax.Value.pair _ _ => false | Compiler.TargetSyntax.Value.adt _ _ => false | Compiler.TargetSyntax.Value.closure rightFunction rightCaptures => ((Nat.beq (leftFunction) (rightFunction)) && valuesEq (leftCaptures) (rightCaptures)))
termination_by structural left _ => left

@[expose] public def valuesEq : (left : List (Compiler.TargetSyntax.Value)) -> (right : List (Compiler.TargetSyntax.Value)) -> Bool
  | List.nil, right => (match right with | List.nil => true | List.cons _ _ => false)
  | List.cons leftHead leftTail, right => (match right with | List.nil => false | List.cons rightHead rightTail => (valueEq (leftHead) (rightHead) && valuesEq (leftTail) (rightTail)))
termination_by structural left _ => left
end

mutual
@[expose] public def exprSize : (expression : Compiler.TargetSyntax.Expr) -> Nat
  | Compiler.TargetSyntax.Expr.value _ _ => 1
  | Compiler.TargetSyntax.Expr.var _ => 1
  | Compiler.TargetSyntax.Expr.«let» _ _ bound body => (1 + (exprSize (bound) + exprSize (body)))
  | Compiler.TargetSyntax.Expr.cond condition thenBranch elseBranch => (1 + (exprSize (condition) + (exprSize (thenBranch) + exprSize (elseBranch))))
  | Compiler.TargetSyntax.Expr.«match» _ scrutinee arms => (1 + (exprSize (scrutinee) + armsSize (arms)))
  | Compiler.TargetSyntax.Expr.build _ _ operands => (1 + exprsSize (operands))
  | Compiler.TargetSyntax.Expr.call _ operands => (1 + exprsSize (operands))
  | Compiler.TargetSyntax.Expr.closure _ operands => (1 + exprsSize (operands))
  | Compiler.TargetSyntax.Expr.apply target operands => (1 + (exprSize (target) + exprsSize (operands)))
  | Compiler.TargetSyntax.Expr.prim _ operands => (1 + exprsSize (operands))
  | Compiler.TargetSyntax.Expr.first inner => (1 + exprSize (inner))
  | Compiler.TargetSyntax.Expr.second inner => (1 + exprSize (inner))
  | Compiler.TargetSyntax.Expr.field inner _ => (1 + exprSize (inner))
termination_by structural expression => expression

@[expose] public def exprsSize : (expressions : List (Compiler.TargetSyntax.Expr)) -> Nat
  | List.nil => 0
  | List.cons head rest => (exprSize (head) + exprsSize (rest))
termination_by structural expressions => expressions

@[expose] public def armsSize : (arms : List (Compiler.TargetSyntax.Arm)) -> Nat
  | List.nil => 0
  | List.cons head rest => (armSize (head) + armsSize (rest))
termination_by structural arms => arms

@[expose] public def armSize : (arm : Compiler.TargetSyntax.Arm) -> Nat
  | Compiler.TargetSyntax.Arm.arm _ _ body => (1 + exprSize (body))
termination_by structural arm => arm
end

@[expose] public def functionsSize : (functions : List (Compiler.TargetSyntax.Function)) -> Nat
  | List.nil => 0
  | List.cons head rest => (exprSize ((head).body) + functionsSize (rest))

@[expose] public def statusOn : (system : Compiler.TargetSyntax.Program) -> (reference : Compiler.TargetSyntax.Program) -> (fuel : Nat) -> (domain : List (Compiler.TargetSyntax.Value)) -> Status
  | _system, _reference, _fuel, List.nil => Status.admitted (0) (0)
  | system, reference, fuel, List.cons argument rest => (match Compiler.TargetSemantics.run (fuel) (system) (0) ((argument :: ([] : List (Compiler.TargetSyntax.Value)))) with | Compiler.TargetSemantics.Outcome.value produced steps => (match Compiler.TargetSemantics.run (fuel) (reference) (0) ((argument :: ([] : List (Compiler.TargetSyntax.Value)))) with | Compiler.TargetSemantics.Outcome.value expected _ => (if valueEq (produced) (expected) then (match statusOn (system) (reference) (fuel) (rest) with | Status.admitted restSteps size => Status.admitted ((steps + restSteps)) (size) | Status.inadmissible => Status.inadmissible | Status.unresolved => Status.unresolved) else Status.inadmissible) | Compiler.TargetSemantics.Outcome.overflow _ => Status.unresolved | Compiler.TargetSemantics.Outcome.stuck => Status.unresolved | Compiler.TargetSemantics.Outcome.exhausted => Status.unresolved) | Compiler.TargetSemantics.Outcome.overflow _ => Status.inadmissible | Compiler.TargetSemantics.Outcome.stuck => Status.inadmissible | Compiler.TargetSemantics.Outcome.exhausted => Status.unresolved)

@[expose] public def status (request : Request) (grammar : Grammar) (selector : Selector) : Status := (let system : Compiler.TargetSyntax.Program := realize (grammar) (selector); (match statusOn (system) ((request).reference) (((request).machine).fuel) ((request).domain) with | Status.admitted steps _ => Status.admitted ((steps + (LexLeanRuntime.multiply ((LexLeanRuntime.length ((request).domain) : Nat)) (preparationCharge (((request).machine).actions)) : Nat))) (functionsSize ((system).functions)) | Status.inadmissible => Status.inadmissible | Status.unresolved => Status.unresolved))

@[expose] public def statuses : (request : Request) -> (grammar : Grammar) -> (selectors : List (Selector)) -> (next : Nat) -> List ((Prod (Nat) (Status)))
  | _request, _grammar, List.nil, _next => ([] : List ((Prod (Nat) (Status))))
  | request, grammar, List.cons selector rest, next => ((next, status (request) (grammar) (selector)) :: statuses (request) (grammar) (rest) ((next + 1)))

@[expose] public def anyUnresolved : (entries : List ((Prod (Nat) (Status)))) -> Bool
  | List.nil => false
  | List.cons entry rest => (match (entry).2 with | Status.unresolved => true | Status.admitted _ _ => anyUnresolved (rest) | Status.inadmissible => anyUnresolved (rest))

@[expose] public def admitted : (entries : List ((Prod (Nat) (Status)))) -> List ((Prod (Nat) ((Prod (Nat) (Nat)))))
  | List.nil => ([] : List ((Prod (Nat) ((Prod (Nat) (Nat))))))
  | List.cons entry rest => (match (entry).2 with | Status.admitted steps size => (((entry).1, (steps, size)) :: admitted (rest)) | Status.inadmissible => admitted (rest) | Status.unresolved => admitted (rest))

@[expose] public def minimumSteps : (costed : List ((Prod (Nat) ((Prod (Nat) (Nat)))))) -> (best : Nat) -> Nat
  | List.nil, best => best
  | List.cons entry rest, best => minimumSteps (rest) ((if (Nat.blt (((entry).2).1) (best)) then ((entry).2).1 else best))

@[expose] public def withSteps : (costed : List ((Prod (Nat) ((Prod (Nat) (Nat)))))) -> (value : Nat) -> List (Nat)
  | List.nil, _value => ([] : List (Nat))
  | List.cons entry rest, value => (if (Nat.beq (((entry).2).1) (value)) then ((entry).1 :: withSteps (rest) (value)) else withSteps (rest) (value))

@[expose] public def dominates (left : (Prod (Nat) (Nat))) (right : (Prod (Nat) (Nat))) : Bool := (((Nat.ble ((left).1) ((right).1)) && (Nat.ble ((left).2) ((right).2))) && ((Nat.blt ((left).1) ((right).1)) || (Nat.blt ((left).2) ((right).2))))

@[expose] public def dominated : (cost : (Prod (Nat) (Nat))) -> (others : List ((Prod (Nat) ((Prod (Nat) (Nat)))))) -> Bool
  | _cost, List.nil => false
  | cost, List.cons other rest => (dominates ((other).2) (cost) || dominated (cost) (rest))

@[expose] public def nondominated : (candidates : List ((Prod (Nat) ((Prod (Nat) (Nat)))))) -> (all : List ((Prod (Nat) ((Prod (Nat) (Nat)))))) -> List (Nat)
  | List.nil, _all => ([] : List (Nat))
  | List.cons entry rest, all => (if dominated ((entry).2) (all) then nondominated (rest) (all) else ((entry).1 :: nondominated (rest) (all)))

@[expose] public def evaluate (request : Request) (grammar : Grammar) : Answer := (let entries : List ((Prod (Nat) (Status))) := statuses (request) (grammar) (expand (grammar)) (0); (if anyUnresolved (entries) then Answer.incomplete else (let costed : List ((Prod (Nat) ((Prod (Nat) (Nat))))) := admitted (entries); (match costed with | List.nil => Answer.infeasible | List.cons head _ => (match (request).objective with | Objective.scalar => (let best : Nat := minimumSteps (costed) (((head).2).1); Answer.argmin (withSteps (costed) (best)) (best)) | Objective.vector => Answer.frontier (nondominated (costed) (costed)))))))

@[expose] public def answer (request : Request) : Answer := (match validate (request) with | Option.none => (match (request).carrier with | Carrier.grammar grammar => evaluate (request) (grammar) | Carrier.internalPlans => Answer.incomplete | Carrier.optimizerOutput => Answer.incomplete | Carrier.discovered _ => Answer.incomplete | Carrier.cached => Answer.incomplete) | Option.some rejection => Answer.rejected (rejection))

@[expose] public def weaklyBelow : (left : List (Nat)) -> (right : List (Nat)) -> Bool
  | List.nil, _right => true
  | List.cons leftHead leftRest, right => (match right with | List.nil => false | List.cons rightHead rightRest => ((Nat.ble (leftHead) (rightHead)) && weaklyBelow (leftRest) (rightRest)))

@[expose] public def strictlyBelow (left : List (Nat)) (right : List (Nat)) : Bool := (weaklyBelow (left) (right) && (!weaklyBelow (right) (left)))

@[expose] public def tableDominated : (cost : List (Nat)) -> (table : List ((Prod (Nat) (List (Nat))))) -> Bool
  | _cost, List.nil => false
  | cost, List.cons row rest => (strictlyBelow ((row).2) (cost) || tableDominated (cost) (rest))

@[expose] public def tableFrontierFrom : (rows : List ((Prod (Nat) (List (Nat))))) -> (table : List ((Prod (Nat) (List (Nat))))) -> List (Nat)
  | List.nil, _table => ([] : List (Nat))
  | List.cons row rest, table => (if tableDominated ((row).2) (table) then tableFrontierFrom (rest) (table) else ((row).1 :: tableFrontierFrom (rest) (table)))

@[expose] public def tableFrontier (table : List ((Prod (Nat) (List (Nat))))) : List (Nat) := tableFrontierFrom (table) (table)

@[expose] public def sameIds : (left : List (Nat)) -> (right : List (Nat)) -> Bool
  | List.nil, right => (match right with | List.nil => true | List.cons _ _ => false)
  | List.cons leftHead leftRest, right => (match right with | List.nil => false | List.cons rightHead rightRest => ((Nat.beq (leftHead) (rightHead)) && sameIds (leftRest) (rightRest)))

@[expose] public def tableAttains : (table : List ((Prod (Nat) (List (Nat))))) -> (cost : List (Nat)) -> Bool
  | List.nil, _cost => false
  | List.cons row rest, cost => ((weaklyBelow ((row).2) (cost) && weaklyBelow (cost) ((row).2)) || tableAttains (rest) (cost))

@[expose] public def rowMinimum : (table : List ((Prod (Nat) (List (Nat))))) -> (best : Nat) -> Nat
  | List.nil, best => best
  | List.cons row rest, best => rowMinimum (rest) ((match (row).2 with | List.nil => best | List.cons cost _ => (if (Nat.blt (cost) (best)) then cost else best)))

@[expose] public def rewriteOnce : (rules : List ((Prod (Nat) (Nat)))) -> (item : Nat) -> Option (Nat)
  | List.nil, _item => Option.none
  | List.cons rule rest, item => (if (Nat.beq ((rule).1) (item)) then Option.some ((rule).2) else rewriteOnce (rest) (item))

@[expose] public def normalize : (fuel : Nat) -> (rules : List ((Prod (Nat) (Nat)))) -> (item : Nat) -> Nat
  | Nat.zero, _rules, item => item
  | Nat.succ remaining, rules, item => (match rewriteOnce (rules) (item) with | Option.none => item | Option.some next => normalize (remaining) (rules) (next))

@[expose] public def listMax : (costs : List (Nat)) -> (worst : Nat) -> Nat
  | List.nil, worst => worst
  | List.cons cost rest, worst => listMax (rest) ((if (Nat.blt (worst) (cost)) then cost else worst))

@[expose] public def uniformWorst : (table : List ((Prod (Nat) (List (Nat))))) -> (best : Nat) -> Nat
  | List.nil, best => best
  | List.cons row rest, best => uniformWorst (rest) ((let worst : Nat := listMax ((row).2) (0); (if (Nat.blt (worst) (best)) then worst else best)))

public theorem vec02Frontier : (tableFrontier (((0, (1 :: (3 :: ([] : List (Nat))))) :: ((1, (2 :: (2 :: ([] : List (Nat))))) :: ((2, (3 :: (1 :: ([] : List (Nat))))) :: ((3, (3 :: (3 :: ([] : List (Nat))))) :: ([] : List ((Prod (Nat) (List (Nat)))))))))) = (0 :: (1 :: (2 :: ([] : List (Nat)))))) := by
  decide

public theorem rej14ComponentwiseMinimaUnattained : (tableAttains (((0, (1 :: (3 :: ([] : List (Nat))))) :: ((1, (2 :: (2 :: ([] : List (Nat))))) :: ((2, (3 :: (1 :: ([] : List (Nat))))) :: ((3, (3 :: (3 :: ([] : List (Nat))))) :: ([] : List ((Prod (Nat) (List (Nat)))))))))) ((1 :: (1 :: ([] : List (Nat))))) = false) := by
  decide

public theorem rej29FrontierOmission : (sameIds (tableFrontier (((0, (1 :: (3 :: ([] : List (Nat))))) :: ((1, (2 :: (2 :: ([] : List (Nat))))) :: ((2, (3 :: (1 :: ([] : List (Nat))))) :: ((3, (3 :: (3 :: ([] : List (Nat))))) :: ([] : List ((Prod (Nat) (List (Nat))))))))))) ((0 :: (1 :: ([] : List (Nat))))) = false) := by
  decide

public theorem vec01Optimum : (rowMinimum (((0, (5 :: ([] : List (Nat)))) :: ((1, (6 :: ([] : List (Nat)))) :: ([] : List ((Prod (Nat) (List (Nat)))))))) (1000) = 5) := by
  decide

public theorem vec01Extension : (rowMinimum (((0, (5 :: ([] : List (Nat)))) :: ((1, (6 :: ([] : List (Nat)))) :: ((2, (4 :: ([] : List (Nat)))) :: ([] : List ((Prod (Nat) (List (Nat))))))))) (1000) = 4) := by
  decide

public theorem vec04NormalFormIsB : (normalize (10) (((0, 1) :: ([] : List ((Prod (Nat) (Nat)))))) (0) = 1) := by
  decide

public theorem vec04GlobalMinimumIsC : (tableFrontier (((0, (2 :: ([] : List (Nat)))) :: ((1, (1 :: ([] : List (Nat)))) :: ((2, (0 :: ([] : List (Nat)))) :: ([] : List ((Prod (Nat) (List (Nat))))))))) = (2 :: ([] : List (Nat)))) := by
  decide

public theorem vec17EnvelopeAtZero : (rowMinimum (((0, (0 :: ([] : List (Nat)))) :: ((1, (10 :: ([] : List (Nat)))) :: ([] : List ((Prod (Nat) (List (Nat)))))))) (1000) = 0) := by
  decide

public theorem vec17EnvelopeAtOne : (rowMinimum (((0, (10 :: ([] : List (Nat)))) :: ((1, (0 :: ([] : List (Nat)))) :: ([] : List ((Prod (Nat) (List (Nat)))))))) (1000) = 0) := by
  decide

public theorem vec17UniformWorstCase : (uniformWorst (((0, (0 :: (10 :: ([] : List (Nat))))) :: ((1, (10 :: (0 :: ([] : List (Nat))))) :: ([] : List ((Prod (Nat) (List (Nat)))))))) (1000) = 10) := by
  decide

end Compiler.Gnaf

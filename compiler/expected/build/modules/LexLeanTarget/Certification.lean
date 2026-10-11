module
public import Init
public import LexLeanTarget.Gnaf
public import LexLeanTarget.TargetSemantics
public import LexLeanTarget.TargetSyntax
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
set_option linter.constructorNameAsVariable false
namespace LexLeanTarget.Certification

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

public inductive Schema where
  | collapse
  | specializeSmall
  | specializeLarge

public inductive Premise where
  | excluded (_ : Nat)
  | paysAtLeast (_ : List (Nat))
  | measured
  | reduced (_ : Schema)

public structure Bridge where
  slope : Nat
  offset : Nat

public inductive Route where
  | lowerBound (_ : Nat) (_ : List (Nat))
  | frontierCover (_ : List (Nat))
  | reduction
  | representation (_ : Nat) (_ : Option (Bridge))
  | normality (_ : Nat)

public structure Certificate where
  verifier : String
  «universe» : String
  objective : LexLeanTarget.Gnaf.Objective
  claim : LexLeanTarget.Gnaf.ClaimClass
  scope : LexLeanTarget.Gnaf.Scope
  route : Route
  premises : List (Premise)

public inductive Fact where
  | inadmissible
  | admitted (_ : Nat) (_ : Nat)
  | atLeast (_ : Nat) (_ : Nat)
  | reduces (_ : Nat)
  | unresolved
  | unproven

public inductive Refusal where
  | verifierMismatch
  | requestRejected (_ : LexLeanTarget.Gnaf.Rejection)
  | universeMismatch
  | objectiveMismatch
  | claimMismatch
  | scopeMismatch
  | routeIncompatible
  | missingPremise (_ : Nat)
  | extraPremise (_ : Nat)

public inductive Obligation where
  | unresolvedMember (_ : Nat)
  | memberUnproven (_ : Nat)
  | memberBelowBound (_ : Nat)
  | attainerOmitted (_ : Nat)
  | attainerUnmeasured (_ : Nat)
  | attainerInadmissible (_ : Nat)
  | attainerAboveBound (_ : Nat)
  | attainersMalformed
  | boundUnattained
  | frontierMemberUnmeasured (_ : Nat)
  | frontierMemberDominated (_ : Nat)
  | frontierMemberOmitted (_ : Nat)
  | reductionIncomplete (_ : Nat)
  | noAdmittedMember
  | bridgeMissing
  | bridgeViolated (_ : Nat)
  | bridgeUnproved (_ : Nat)
  | normalOutOfRange
  | normalNotMinimal (_ : Nat)
  | normalUnmeasured
  | optimalityUnproved
  | systemNotNormal

public inductive Established where
  | notCertified
  | correctOnly (_ : Nat) (_ : Nat) (_ : Nat)
  | locallyNormalized (_ : Nat)
  | representationMinimal (_ : Nat) (_ : Nat)

public inductive Certified where
  | optimum (_ : List (Nat)) (_ : Nat)
  | argmin (_ : List (Nat)) (_ : Nat)
  | frontier (_ : List (Nat))

public inductive Verdict where
  | refused (_ : Refusal)
  | certified (_ : Certified)
  | incomplete (_ : Obligation) (_ : Established)

@[expose] public def positionOf : (selectors : List (LexLeanTarget.Gnaf.Selector)) -> (selector : LexLeanTarget.Gnaf.Selector) -> (next : Nat) -> Option (Nat)
  | List.nil, _selector, _next => Option.none
  | List.cons head rest, selector, next => (if LexLeanTarget.Gnaf.selectorEq (selector) (head) then Option.some (next) else positionOf (rest) (selector) ((next + 1)))

@[expose] public def selectorAt : (items : List (LexLeanTarget.Gnaf.Selector)) -> (index : Nat) -> Option (LexLeanTarget.Gnaf.Selector)
  | List.nil, _index => Option.none
  | List.cons head rest, index => (match index with | Nat.zero => Option.some (head) | Nat.succ remaining => selectorAt (rest) (remaining))

@[expose] public def valueAt : (items : List (LexLeanTarget.TargetSyntax.Value)) -> (index : Nat) -> Option (LexLeanTarget.TargetSyntax.Value)
  | List.nil, _index => Option.none
  | List.cons head rest, index => (match index with | Nat.zero => Option.some (head) | Nat.succ remaining => valueAt (rest) (remaining))

@[expose] public def factAt : (items : List (Fact)) -> (index : Nat) -> Option (Fact)
  | List.nil, _index => Option.none
  | List.cons head rest, index => (match index with | Nat.zero => Option.some (head) | Nat.succ remaining => factAt (rest) (remaining))

@[expose] public def natAt : (items : List (Nat)) -> (index : Nat) -> Option (Nat)
  | List.nil, _index => Option.none
  | List.cons head rest, index => (match index with | Nat.zero => Option.some (head) | Nat.succ remaining => natAt (rest) (remaining))

@[expose] public def pickFrom : (values : List (LexLeanTarget.TargetSyntax.Value)) -> (index : Nat) -> (positions : List (Nat)) -> List (LexLeanTarget.TargetSyntax.Value)
  | List.nil, _index, _positions => ([] : List (LexLeanTarget.TargetSyntax.Value))
  | List.cons head rest, index, positions => (match positions with | List.nil => ([] : List (LexLeanTarget.TargetSyntax.Value)) | List.cons wanted later => (if (Nat.beq (wanted) (index)) then (head :: pickFrom (rest) ((index + 1)) (later)) else pickFrom (rest) ((index + 1)) (positions)))

@[expose] public def ascending : (positions : List (Nat)) -> (lower : Nat) -> Bool
  | List.nil, _lower => true
  | List.cons head rest, lower => ((Nat.ble (lower) (head)) && ascending (rest) ((head + 1)))

@[expose] public def allBelow : (positions : List (Nat)) -> (bound : Nat) -> Bool
  | List.nil, _bound => true
  | List.cons head rest, bound => ((Nat.blt (head) (bound)) && allBelow (rest) (bound))

@[expose] public def isList (value : LexLeanTarget.TargetSyntax.Value) (_limit : Nat) : Bool := (match value with | LexLeanTarget.TargetSyntax.Value.unit => false | LexLeanTarget.TargetSyntax.Value.bool _ => false | LexLeanTarget.TargetSyntax.Value.nat _ => false | LexLeanTarget.TargetSyntax.Value.int _ => false | LexLeanTarget.TargetSyntax.Value.u8 _ => false | LexLeanTarget.TargetSyntax.Value.u16 _ => false | LexLeanTarget.TargetSyntax.Value.u32 _ => false | LexLeanTarget.TargetSyntax.Value.u64 _ => false | LexLeanTarget.TargetSyntax.Value.i8 _ => false | LexLeanTarget.TargetSyntax.Value.i16 _ => false | LexLeanTarget.TargetSyntax.Value.i32 _ => false | LexLeanTarget.TargetSyntax.Value.i64 _ => false | LexLeanTarget.TargetSyntax.Value.string _ => false | LexLeanTarget.TargetSyntax.Value.bytes _ => false | LexLeanTarget.TargetSyntax.Value.ordering _ => false | LexLeanTarget.TargetSyntax.Value.none => false | LexLeanTarget.TargetSyntax.Value.some _ => false | LexLeanTarget.TargetSyntax.Value.ok _ => false | LexLeanTarget.TargetSyntax.Value.error _ => false | LexLeanTarget.TargetSyntax.Value.list _ => true | LexLeanTarget.TargetSyntax.Value.pair _ _ => false | LexLeanTarget.TargetSyntax.Value.adt _ _ => false | LexLeanTarget.TargetSyntax.Value.closure _ _ => false)

@[expose] public def isShorter (value : LexLeanTarget.TargetSyntax.Value) (limit : Nat) : Bool := (match value with | LexLeanTarget.TargetSyntax.Value.unit => false | LexLeanTarget.TargetSyntax.Value.bool _ => false | LexLeanTarget.TargetSyntax.Value.nat _ => false | LexLeanTarget.TargetSyntax.Value.int _ => false | LexLeanTarget.TargetSyntax.Value.u8 _ => false | LexLeanTarget.TargetSyntax.Value.u16 _ => false | LexLeanTarget.TargetSyntax.Value.u32 _ => false | LexLeanTarget.TargetSyntax.Value.u64 _ => false | LexLeanTarget.TargetSyntax.Value.i8 _ => false | LexLeanTarget.TargetSyntax.Value.i16 _ => false | LexLeanTarget.TargetSyntax.Value.i32 _ => false | LexLeanTarget.TargetSyntax.Value.i64 _ => false | LexLeanTarget.TargetSyntax.Value.string _ => false | LexLeanTarget.TargetSyntax.Value.bytes _ => false | LexLeanTarget.TargetSyntax.Value.ordering _ => false | LexLeanTarget.TargetSyntax.Value.none => false | LexLeanTarget.TargetSyntax.Value.some _ => false | LexLeanTarget.TargetSyntax.Value.ok _ => false | LexLeanTarget.TargetSyntax.Value.error _ => false | LexLeanTarget.TargetSyntax.Value.list items => (Nat.blt ((LexLeanRuntime.length (items) : Nat)) (limit)) | LexLeanTarget.TargetSyntax.Value.pair _ _ => false | LexLeanTarget.TargetSyntax.Value.adt _ _ => false | LexLeanTarget.TargetSyntax.Value.closure _ _ => false)

@[expose] public def isLong (value : LexLeanTarget.TargetSyntax.Value) (limit : Nat) : Bool := (match value with | LexLeanTarget.TargetSyntax.Value.unit => false | LexLeanTarget.TargetSyntax.Value.bool _ => false | LexLeanTarget.TargetSyntax.Value.nat _ => false | LexLeanTarget.TargetSyntax.Value.int _ => false | LexLeanTarget.TargetSyntax.Value.u8 _ => false | LexLeanTarget.TargetSyntax.Value.u16 _ => false | LexLeanTarget.TargetSyntax.Value.u32 _ => false | LexLeanTarget.TargetSyntax.Value.u64 _ => false | LexLeanTarget.TargetSyntax.Value.i8 _ => false | LexLeanTarget.TargetSyntax.Value.i16 _ => false | LexLeanTarget.TargetSyntax.Value.i32 _ => false | LexLeanTarget.TargetSyntax.Value.i64 _ => false | LexLeanTarget.TargetSyntax.Value.string _ => false | LexLeanTarget.TargetSyntax.Value.bytes _ => false | LexLeanTarget.TargetSyntax.Value.ordering _ => false | LexLeanTarget.TargetSyntax.Value.none => false | LexLeanTarget.TargetSyntax.Value.some _ => false | LexLeanTarget.TargetSyntax.Value.ok _ => false | LexLeanTarget.TargetSyntax.Value.error _ => false | LexLeanTarget.TargetSyntax.Value.list items => (Nat.ble (limit) ((LexLeanRuntime.length (items) : Nat))) | LexLeanTarget.TargetSyntax.Value.pair _ _ => false | LexLeanTarget.TargetSyntax.Value.adt _ _ => false | LexLeanTarget.TargetSyntax.Value.closure _ _ => false)

@[expose] public def allLists : (values : List (LexLeanTarget.TargetSyntax.Value)) -> (limit : Nat) -> Bool
  | List.nil, _limit => true
  | List.cons head rest, limit => (isList (head) (limit) && allLists (rest) (limit))

@[expose] public def allShorter : (values : List (LexLeanTarget.TargetSyntax.Value)) -> (limit : Nat) -> Bool
  | List.nil, _limit => true
  | List.cons head rest, limit => (isShorter (head) (limit) && allShorter (rest) (limit))

@[expose] public def allLong : (values : List (LexLeanTarget.TargetSyntax.Value)) -> (limit : Nat) -> Bool
  | List.nil, _limit => true
  | List.cons head rest, limit => (isLong (head) (limit) && allLong (rest) (limit))

mutual
@[expose] public def exprIsolated : (expression : LexLeanTarget.TargetSyntax.Expr) -> Bool
  | LexLeanTarget.TargetSyntax.Expr.value _ _ => true
  | LexLeanTarget.TargetSyntax.Expr.var _ => true
  | LexLeanTarget.TargetSyntax.Expr.«let» _ _ bound body => (true && (exprIsolated (bound) && exprIsolated (body)))
  | LexLeanTarget.TargetSyntax.Expr.cond condition thenBranch elseBranch => (true && (exprIsolated (condition) && (exprIsolated (thenBranch) && exprIsolated (elseBranch))))
  | LexLeanTarget.TargetSyntax.Expr.«match» _ scrutinee arms => (true && (exprIsolated (scrutinee) && armsIsolated (arms)))
  | LexLeanTarget.TargetSyntax.Expr.build _ _ operands => (true && exprsIsolated (operands))
  | LexLeanTarget.TargetSyntax.Expr.call function operands => ((!(Nat.beq (function) (0))) && exprsIsolated (operands))
  | LexLeanTarget.TargetSyntax.Expr.closure _ operands => (true && exprsIsolated (operands))
  | LexLeanTarget.TargetSyntax.Expr.apply target operands => (false && (exprIsolated (target) && exprsIsolated (operands)))
  | LexLeanTarget.TargetSyntax.Expr.prim _ operands => (true && exprsIsolated (operands))
  | LexLeanTarget.TargetSyntax.Expr.first inner => (true && exprIsolated (inner))
  | LexLeanTarget.TargetSyntax.Expr.second inner => (true && exprIsolated (inner))
  | LexLeanTarget.TargetSyntax.Expr.field inner _ => (true && exprIsolated (inner))
termination_by structural expression => expression

@[expose] public def exprsIsolated : (expressions : List (LexLeanTarget.TargetSyntax.Expr)) -> Bool
  | List.nil => true
  | List.cons head rest => (exprIsolated (head) && exprsIsolated (rest))
termination_by structural expressions => expressions

@[expose] public def armsIsolated : (arms : List (LexLeanTarget.TargetSyntax.Arm)) -> Bool
  | List.nil => true
  | List.cons head rest => (armIsolated (head) && armsIsolated (rest))
termination_by structural arms => arms

@[expose] public def armIsolated : (arm : LexLeanTarget.TargetSyntax.Arm) -> Bool
  | LexLeanTarget.TargetSyntax.Arm.arm _ _ body => (true && exprIsolated (body))
termination_by structural arm => arm
end

@[expose] public def plansIsolated : (plans : List (LexLeanTarget.Gnaf.Plan)) -> Bool
  | List.nil => true
  | List.cons plan rest => (exprIsolated (((plan).function).body) && plansIsolated (rest))

@[expose] public def rewrite (schema : Schema) (selector : LexLeanTarget.Gnaf.Selector) : Option (LexLeanTarget.Gnaf.Selector) := (match schema with | Schema.collapse => (match selector with | LexLeanTarget.Gnaf.Selector.fixed _ => Option.none | LexLeanTarget.Gnaf.Selector.dispatch _ small large => (if (Nat.beq (small) (large)) then Option.some (LexLeanTarget.Gnaf.Selector.fixed (small)) else Option.none)) | Schema.specializeSmall => (match selector with | LexLeanTarget.Gnaf.Selector.fixed _ => Option.none | LexLeanTarget.Gnaf.Selector.dispatch _ small _ => Option.some (LexLeanTarget.Gnaf.Selector.fixed (small))) | Schema.specializeLarge => (match selector with | LexLeanTarget.Gnaf.Selector.fixed _ => Option.none | LexLeanTarget.Gnaf.Selector.dispatch _ _ large => Option.some (LexLeanTarget.Gnaf.Selector.fixed (large))))

@[expose] public def eligible (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) (schema : Schema) : Bool := (plansIsolated ((grammar).plans) && (match selector with | LexLeanTarget.Gnaf.Selector.fixed _ => false | LexLeanTarget.Gnaf.Selector.dispatch threshold small large => (match schema with | Schema.collapse => ((Nat.beq (small) (large)) && allLists ((request).domain) (0)) | Schema.specializeSmall => allShorter ((request).domain) (threshold) | Schema.specializeLarge => allLong ((request).domain) (threshold))))

@[expose] public def staticSize (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) : Nat := (let system : LexLeanTarget.TargetSyntax.Program := LexLeanTarget.Gnaf.realize (grammar) (selector); LexLeanTarget.Gnaf.reachableSize ((system).functions) (LexLeanTarget.Gnaf.reachable ((system).functions)))

@[expose] public def staticCharge (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) : Nat := (let system : LexLeanTarget.TargetSyntax.Program := LexLeanTarget.Gnaf.realize (grammar) (selector); (LexLeanRuntime.multiply ((LexLeanRuntime.length ((request).domain) : Nat)) (LexLeanTarget.Gnaf.reachableCharge (((request).machine).actions) ((grammar).plans) (LexLeanTarget.Gnaf.reachable ((system).functions))) : Nat))

@[expose] public def improves (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) (schema : Schema) : Bool := (match rewrite (schema) (selector) with | Option.none => false | Option.some target => (eligible (request) (grammar) (selector) (schema) && ((Nat.ble (staticSize (grammar) (target)) (staticSize (grammar) (selector))) && (Nat.ble (staticCharge (request) (grammar) (target)) (staticCharge (request) (grammar) (selector))))))

@[expose] public def isNormal (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) : Bool := ((!improves (request) (grammar) (selector) (Schema.collapse)) && ((!improves (request) (grammar) (selector) (Schema.specializeSmall)) && (!improves (request) (grammar) (selector) (Schema.specializeLarge))))

@[expose] public def premiseFact (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selectors : List (LexLeanTarget.Gnaf.Selector)) (selector : LexLeanTarget.Gnaf.Selector) (premise : Premise) : Fact := (let system : LexLeanTarget.TargetSyntax.Program := LexLeanTarget.Gnaf.realize (grammar) (selector); (match premise with | Premise.excluded position => (match valueAt ((request).domain) (position) with | Option.none => Fact.unproven | Option.some argument => (match LexLeanTarget.Gnaf.statusOn (system) ((request).reference) (((request).machine).fuel) ((argument :: ([] : List (LexLeanTarget.TargetSyntax.Value)))) with | LexLeanTarget.Gnaf.Status.admitted _ _ => Fact.unproven | LexLeanTarget.Gnaf.Status.inadmissible => Fact.inadmissible | LexLeanTarget.Gnaf.Status.unresolved => Fact.unproven)) | Premise.paysAtLeast positions => (let picked : List (LexLeanTarget.TargetSyntax.Value) := pickFrom ((request).domain) (0) (positions); (if ((Nat.beq ((LexLeanRuntime.length (picked) : Nat)) ((LexLeanRuntime.length (positions) : Nat))) && ascending (positions) (0)) then (match LexLeanTarget.Gnaf.statusOn (system) ((request).reference) (((request).machine).fuel) (picked) with | LexLeanTarget.Gnaf.Status.admitted steps _ => Fact.atLeast ((steps + staticCharge (request) (grammar) (selector))) (staticSize (grammar) (selector)) | LexLeanTarget.Gnaf.Status.inadmissible => Fact.unproven | LexLeanTarget.Gnaf.Status.unresolved => Fact.unproven) else Fact.unproven)) | Premise.measured => (match LexLeanTarget.Gnaf.status (request) (grammar) (selector) with | LexLeanTarget.Gnaf.Status.admitted cost size => Fact.admitted (cost) (size) | LexLeanTarget.Gnaf.Status.inadmissible => Fact.inadmissible | LexLeanTarget.Gnaf.Status.unresolved => Fact.unresolved) | Premise.reduced schema => (match rewrite (schema) (selector) with | Option.none => Fact.unproven | Option.some target => (match positionOf (selectors) (target) (0) with | Option.none => Fact.unproven | Option.some targetPosition => (if improves (request) (grammar) (selector) (schema) then Fact.reduces (targetPosition) else Fact.unproven)))))

@[expose] public def factsOf : (request : LexLeanTarget.Gnaf.Request) -> (grammar : LexLeanTarget.Gnaf.Grammar) -> (all : List (LexLeanTarget.Gnaf.Selector)) -> (selectors : List (LexLeanTarget.Gnaf.Selector)) -> (premises : List (Premise)) -> List (Fact)
  | _request, _grammar, _all, List.nil, _premises => ([] : List (Fact))
  | request, grammar, all, List.cons selector rest, premises => (match premises with | List.nil => (Fact.unproven :: factsOf (request) (grammar) (all) (rest) (premises)) | List.cons premise later => (premiseFact (request) (grammar) (all) (selector) (premise) :: factsOf (request) (grammar) (all) (rest) (later)))

@[expose] public def sizesOf : (grammar : LexLeanTarget.Gnaf.Grammar) -> (selectors : List (LexLeanTarget.Gnaf.Selector)) -> List (Nat)
  | _grammar, List.nil => ([] : List (Nat))
  | grammar, List.cons selector rest => (staticSize (grammar) (selector) :: sizesOf (grammar) (rest))

@[expose] public def factRows : (facts : List (Fact)) -> (position : Nat) -> List ((Prod (Nat) (List (Nat))))
  | List.nil, _position => ([] : List ((Prod (Nat) (List (Nat)))))
  | List.cons head rest, position => (match head with | Fact.inadmissible => factRows (rest) ((position + 1)) | Fact.admitted cost size => ((position, (cost :: (size :: ([] : List (Nat))))) :: factRows (rest) ((position + 1))) | Fact.atLeast _ _ => factRows (rest) ((position + 1)) | Fact.reduces _ => factRows (rest) ((position + 1)) | Fact.unresolved => factRows (rest) ((position + 1)) | Fact.unproven => factRows (rest) ((position + 1)))

@[expose] public def rowsOfMembers : (rows : List ((Prod (Nat) (List (Nat))))) -> (members : List (Nat)) -> List ((Prod (Nat) (List (Nat))))
  | List.nil, _members => ([] : List ((Prod (Nat) (List (Nat)))))
  | List.cons row rest, members => (if LexLeanTarget.Gnaf.natIn ((row).1) (members) then (row :: rowsOfMembers (rest) (members)) else rowsOfMembers (rest) (members))

@[expose] public def argumentStatus (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (fuel : Nat) (selector : LexLeanTarget.Gnaf.Selector) (argument : LexLeanTarget.TargetSyntax.Value) : LexLeanTarget.Gnaf.Status := LexLeanTarget.Gnaf.statusOn (LexLeanTarget.Gnaf.realize (grammar) (selector)) ((request).reference) (fuel) ((argument :: ([] : List (LexLeanTarget.TargetSyntax.Value))))

@[expose] public def keepsObservation (before : LexLeanTarget.Gnaf.Status) (after : LexLeanTarget.Gnaf.Status) : Bool := (match before with | LexLeanTarget.Gnaf.Status.admitted _ _ => (match after with | LexLeanTarget.Gnaf.Status.admitted _ _ => true | LexLeanTarget.Gnaf.Status.inadmissible => false | LexLeanTarget.Gnaf.Status.unresolved => false) | LexLeanTarget.Gnaf.Status.inadmissible => true | LexLeanTarget.Gnaf.Status.unresolved => true)

@[expose] public def cheaperObservation (before : LexLeanTarget.Gnaf.Status) (after : LexLeanTarget.Gnaf.Status) : Bool := (match before with | LexLeanTarget.Gnaf.Status.admitted cost _ => (match after with | LexLeanTarget.Gnaf.Status.admitted cheaper _ => (Nat.blt (cheaper) (cost)) | LexLeanTarget.Gnaf.Status.inadmissible => false | LexLeanTarget.Gnaf.Status.unresolved => false) | LexLeanTarget.Gnaf.Status.inadmissible => true | LexLeanTarget.Gnaf.Status.unresolved => true)

@[expose] public def sameObservationsAt : (request : LexLeanTarget.Gnaf.Request) -> (grammar : LexLeanTarget.Gnaf.Grammar) -> (fuel : Nat) -> (selector : LexLeanTarget.Gnaf.Selector) -> (target : LexLeanTarget.Gnaf.Selector) -> (arguments : List (LexLeanTarget.TargetSyntax.Value)) -> Bool
  | _request, _grammar, _fuel, _selector, _target, List.nil => true
  | request, grammar, fuel, selector, target, List.cons argument rest => (keepsObservation (argumentStatus (request) (grammar) (fuel) (selector) (argument)) (argumentStatus (request) (grammar) (fuel) (target) (argument)) && sameObservationsAt (request) (grammar) (fuel) (selector) (target) (rest))

@[expose] public def strictlyCheaperAt : (request : LexLeanTarget.Gnaf.Request) -> (grammar : LexLeanTarget.Gnaf.Grammar) -> (fuel : Nat) -> (selector : LexLeanTarget.Gnaf.Selector) -> (target : LexLeanTarget.Gnaf.Selector) -> (arguments : List (LexLeanTarget.TargetSyntax.Value)) -> Bool
  | _request, _grammar, _fuel, _selector, _target, List.nil => true
  | request, grammar, fuel, selector, target, List.cons argument rest => (cheaperObservation (argumentStatus (request) (grammar) (fuel) (selector) (argument)) (argumentStatus (request) (grammar) (fuel) (target) (argument)) && strictlyCheaperAt (request) (grammar) (fuel) (selector) (target) (rest))

@[expose] public def sameObservations (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) (target : LexLeanTarget.Gnaf.Selector) : Bool := sameObservationsAt (request) (grammar) (((request).machine).fuel) (selector) (target) ((request).domain)

@[expose] public def strictlyCheaper (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) (target : LexLeanTarget.Gnaf.Selector) : Bool := strictlyCheaperAt (request) (grammar) (((request).machine).fuel) (selector) (target) ((request).domain)

@[expose] public def preserves (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selector : LexLeanTarget.Gnaf.Selector) (schema : Schema) : Bool := (match rewrite (schema) (selector) with | Option.none => false | Option.some target => (improves (request) (grammar) (selector) (schema) && (sameObservations (request) (grammar) (selector) (target) && strictlyCheaper (request) (grammar) (selector) (target))))

@[expose] public def pairRelation (request : LexLeanTarget.Gnaf.Request) (first : Nat) (second : Nat) (cheaper : Bool) : Bool := (match (request).carrier with | LexLeanTarget.Gnaf.Carrier.grammar grammar => (let selectors : List (LexLeanTarget.Gnaf.Selector) := LexLeanTarget.Gnaf.expand (grammar); (match selectorAt (selectors) (first) with | Option.none => false | Option.some left => (match selectorAt (selectors) (second) with | Option.none => false | Option.some right => (if cheaper then strictlyCheaper (request) (grammar) (left) (right) else sameObservations (request) (grammar) (left) (right))))) | LexLeanTarget.Gnaf.Carrier.internalPlans => false | LexLeanTarget.Gnaf.Carrier.optimizerOutput => false | LexLeanTarget.Gnaf.Carrier.discovered _ => false | LexLeanTarget.Gnaf.Carrier.cached => false)

@[expose] public def reductionsPreserve : (request : LexLeanTarget.Gnaf.Request) -> (grammar : LexLeanTarget.Gnaf.Grammar) -> (selectors : List (LexLeanTarget.Gnaf.Selector)) -> (premises : List (Premise)) -> Bool
  | _request, _grammar, List.nil, _premises => true
  | request, grammar, List.cons selector rest, premises => (match premises with | List.nil => true | List.cons premise later => ((match premise with | Premise.excluded _ => true | Premise.paysAtLeast _ => true | Premise.measured => true | Premise.reduced schema => preserves (request) (grammar) (selector) (schema)) && reductionsPreserve (request) (grammar) (rest) (later)))

@[expose] public def certificateReductionsPreserve (request : LexLeanTarget.Gnaf.Request) (certificate : Certificate) : Bool := (match (request).carrier with | LexLeanTarget.Gnaf.Carrier.grammar grammar => reductionsPreserve (request) (grammar) (LexLeanTarget.Gnaf.expand (grammar)) ((certificate).premises) | LexLeanTarget.Gnaf.Carrier.internalPlans => false | LexLeanTarget.Gnaf.Carrier.optimizerOutput => false | LexLeanTarget.Gnaf.Carrier.discovered _ => false | LexLeanTarget.Gnaf.Carrier.cached => false)

@[expose] public def lowerPosition (strict : Bool) (bound : Nat) (attainer : Bool) (fact : Fact) (position : Nat) : Option (Obligation) := (match fact with | Fact.inadmissible => (if attainer then Option.some (Obligation.attainerInadmissible (position)) else Option.none) | Fact.admitted cost _ => (if attainer then (if (Nat.beq (cost) (bound)) then Option.none else (if (Nat.blt (cost) (bound)) then Option.some (Obligation.memberBelowBound (position)) else Option.some (Obligation.attainerAboveBound (position)))) else (if (Nat.blt (cost) (bound)) then Option.some (Obligation.memberBelowBound (position)) else (if (strict && (Nat.beq (cost) (bound))) then Option.some (Obligation.attainerOmitted (position)) else Option.none))) | Fact.atLeast cost _ => (if attainer then Option.some (Obligation.attainerUnmeasured (position)) else (if strict then (if (Nat.blt (bound) (cost)) then Option.none else Option.some (Obligation.memberUnproven (position))) else (if (Nat.ble (bound) (cost)) then Option.none else Option.some (Obligation.memberUnproven (position))))) | Fact.reduces _ => (if attainer then Option.some (Obligation.attainerUnmeasured (position)) else Option.none) | Fact.unresolved => Option.some (Obligation.unresolvedMember (position)) | Fact.unproven => Option.some (Obligation.memberUnproven (position)))

@[expose] public def scanLower : (strict : Bool) -> (bound : Nat) -> (attainers : List (Nat)) -> (facts : List (Fact)) -> (position : Nat) -> Option (Obligation)
  | _strict, _bound, _attainers, List.nil, _position => Option.none
  | strict, bound, attainers, List.cons head rest, position => (match lowerPosition (strict) (bound) (LexLeanTarget.Gnaf.natIn (position) (attainers)) (head) (position) with | Option.none => scanLower (strict) (bound) (attainers) (rest) ((position + 1)) | Option.some obligation => Option.some (obligation))

@[expose] public def establishedFrom (facts : List (Fact)) (attainers : List (Nat)) : Established := (match attainers with | List.nil => Established.notCertified | List.cons head _ => (match factAt (facts) (head) with | Option.none => Established.notCertified | Option.some found => (match found with | Fact.inadmissible => Established.notCertified | Fact.admitted cost size => Established.correctOnly (head) (cost) (size) | Fact.atLeast _ _ => Established.notCertified | Fact.reduces _ => Established.notCertified | Fact.unresolved => Established.notCertified | Fact.unproven => Established.notCertified)))

@[expose] public def lowerVerdict (strict : Bool) (bound : Nat) (attainers : List (Nat)) (facts : List (Fact)) : Verdict := (if (!(ascending (attainers) (0) && allBelow (attainers) ((LexLeanRuntime.length (facts) : Nat)))) then Verdict.incomplete (Obligation.attainersMalformed) (Established.notCertified) else (match scanLower (strict) (bound) (attainers) (facts) (0) with | Option.none => (match attainers with | List.nil => Verdict.incomplete (Obligation.boundUnattained) (Established.notCertified) | List.cons _ _ => (if strict then Verdict.certified (Certified.argmin (attainers) (bound)) else Verdict.certified (Certified.optimum (attainers) (bound)))) | Option.some obligation => Verdict.incomplete (obligation) (establishedFrom (facts) (attainers))))

@[expose] public def frontierPosition (member : Bool) (memberRows : List ((Prod (Nat) (List (Nat))))) (fact : Fact) (position : Nat) : Option (Obligation) := (if member then (match fact with | Fact.inadmissible => Option.some (Obligation.frontierMemberUnmeasured (position)) | Fact.admitted cost size => (if LexLeanTarget.Gnaf.dominatedIn (Nat) ((cost :: (size :: ([] : List (Nat))))) (memberRows) then Option.some (Obligation.frontierMemberDominated (position)) else Option.none) | Fact.atLeast _ _ => Option.some (Obligation.frontierMemberUnmeasured (position)) | Fact.reduces _ => Option.some (Obligation.frontierMemberUnmeasured (position)) | Fact.unresolved => Option.some (Obligation.unresolvedMember (position)) | Fact.unproven => Option.some (Obligation.memberUnproven (position))) else (match fact with | Fact.inadmissible => Option.none | Fact.admitted cost size => (if LexLeanTarget.Gnaf.dominatedIn (Nat) ((cost :: (size :: ([] : List (Nat))))) (memberRows) then Option.none else Option.some (Obligation.frontierMemberOmitted (position))) | Fact.atLeast cost size => (if LexLeanTarget.Gnaf.dominatedIn (Nat) ((cost :: (size :: ([] : List (Nat))))) (memberRows) then Option.none else Option.some (Obligation.memberUnproven (position))) | Fact.reduces _ => Option.none | Fact.unresolved => Option.some (Obligation.unresolvedMember (position)) | Fact.unproven => Option.some (Obligation.memberUnproven (position))))

@[expose] public def scanFrontier : (members : List (Nat)) -> (memberRows : List ((Prod (Nat) (List (Nat))))) -> (facts : List (Fact)) -> (position : Nat) -> Option (Obligation)
  | _members, _memberRows, List.nil, _position => Option.none
  | members, memberRows, List.cons head rest, position => (match frontierPosition (LexLeanTarget.Gnaf.natIn (position) (members)) (memberRows) (head) (position) with | Option.none => scanFrontier (members) (memberRows) (rest) ((position + 1)) | Option.some obligation => Option.some (obligation))

@[expose] public def frontierVerdict (members : List (Nat)) (facts : List (Fact)) : Verdict := (let memberRows : List ((Prod (Nat) (List (Nat)))) := rowsOfMembers (factRows (facts) (0)) (members); (if (!(ascending (members) (0) && allBelow (members) ((LexLeanRuntime.length (facts) : Nat)))) then Verdict.incomplete (Obligation.attainersMalformed) (Established.notCertified) else (match scanFrontier (members) (memberRows) (facts) (0) with | Option.none => (match members with | List.nil => Verdict.incomplete (Obligation.noAdmittedMember) (Established.notCertified) | List.cons _ _ => Verdict.certified (Certified.frontier (members))) | Option.some obligation => Verdict.incomplete (obligation) (Established.notCertified))))

@[expose] public def scanReduction : (facts : List (Fact)) -> (position : Nat) -> Option (Obligation)
  | List.nil, _position => Option.none
  | List.cons head rest, position => (match (match head with | Fact.inadmissible => Option.none | Fact.admitted _ _ => Option.none | Fact.atLeast _ _ => Option.some (Obligation.reductionIncomplete (position)) | Fact.reduces _ => Option.none | Fact.unresolved => Option.some (Obligation.unresolvedMember (position)) | Fact.unproven => Option.some (Obligation.reductionIncomplete (position))) with | Option.none => scanReduction (rest) ((position + 1)) | Option.some obligation => Option.some (obligation))

@[expose] public def reductionVerdict (shape : Nat) (facts : List (Fact)) : Verdict := (match scanReduction (facts) (0) with | Option.none => (let rows : List ((Prod (Nat) (List (Nat)))) := factRows (facts) (0); (if (Nat.blt (shape) (3)) then (match LexLeanTarget.Gnaf.argmin (Nat) (LexLeanTarget.Gnaf.scalarRows (rows)) with | Option.none => Verdict.incomplete (Obligation.noAdmittedMember) (Established.notCertified) | Option.some found => lowerVerdict ((Nat.beq (shape) (2))) ((found).2) ((found).1) (facts)) else (match rows with | List.nil => Verdict.incomplete (Obligation.noAdmittedMember) (Established.notCertified) | List.cons _ _ => frontierVerdict (LexLeanTarget.Gnaf.frontier (Nat) (rows)) (facts)))) | Option.some obligation => Verdict.incomplete (obligation) (Established.notCertified))

@[expose] public def bridgePosition (bridge : Bridge) (fact : Fact) (measuredSize : Nat) (position : Nat) : Option (Obligation) := (match fact with | Fact.inadmissible => Option.none | Fact.admitted cost _ => (if (Nat.ble (((LexLeanRuntime.multiply ((bridge).slope) (measuredSize) : Nat) + (bridge).offset)) (cost)) then Option.none else Option.some (Obligation.bridgeViolated (position))) | Fact.atLeast cost _ => (if (Nat.ble (((LexLeanRuntime.multiply ((bridge).slope) (measuredSize) : Nat) + (bridge).offset)) (cost)) then Option.none else Option.some (Obligation.bridgeUnproved (position))) | Fact.reduces _ => Option.some (Obligation.bridgeUnproved (position)) | Fact.unresolved => Option.some (Obligation.unresolvedMember (position)) | Fact.unproven => Option.some (Obligation.bridgeUnproved (position)))

@[expose] public def firstSmaller : (sizes : List (Nat)) -> (limit : Nat) -> (position : Nat) -> Option (Nat)
  | List.nil, _limit, _position => Option.none
  | List.cons head rest, limit, position => (if (Nat.blt (head) (limit)) then Option.some (position) else firstSmaller (rest) (limit) ((position + 1)))

@[expose] public def scanBridge : (bridge : Bridge) -> (facts : List (Fact)) -> (sizes : List (Nat)) -> (position : Nat) -> Option (Obligation)
  | _bridge, List.nil, _sizes, _position => Option.none
  | bridge, List.cons head rest, sizes, position => (match sizes with | List.nil => Option.some (Obligation.bridgeUnproved (position)) | List.cons measuredSize laterSizes => (match bridgePosition (bridge) (head) (measuredSize) (position) with | Option.none => scanBridge (bridge) (rest) (laterSizes) ((position + 1)) | Option.some obligation => Option.some (obligation)))

@[expose] public def representationVerdict (normal : Nat) (bridgeOption : Option (Bridge)) (facts : List (Fact)) (sizes : List (Nat)) : Verdict := (match natAt (sizes) (normal) with | Option.none => Verdict.incomplete (Obligation.normalOutOfRange) (Established.notCertified) | Option.some normalSize => (match firstSmaller (sizes) (normalSize) (0) with | Option.none => (match bridgeOption with | Option.none => Verdict.incomplete (Obligation.bridgeMissing) (Established.representationMinimal (normal) (normalSize)) | Option.some given => (let bridge : Bridge := given; (match scanBridge (bridge) (facts) (sizes) (0) with | Option.none => (let limit : Nat := ((LexLeanRuntime.multiply ((bridge).slope) (normalSize) : Nat) + (bridge).offset); (match factAt (facts) (normal) with | Option.none => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Option.some found => (match found with | Fact.inadmissible => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.admitted cost size => (if (Nat.beq (cost) (limit)) then Verdict.certified (Certified.optimum ((normal :: ([] : List (Nat)))) (cost)) else Verdict.incomplete (Obligation.boundUnattained) (Established.correctOnly (normal) (cost) (size))) | Fact.atLeast _ _ => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.reduces _ => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.unresolved => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.unproven => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified)))) | Option.some obligation => Verdict.incomplete (obligation) (Established.notCertified)))) | Option.some smaller => Verdict.incomplete (Obligation.normalNotMinimal (smaller)) (Established.notCertified)))

@[expose] public def normalityVerdict (request : LexLeanTarget.Gnaf.Request) (grammar : LexLeanTarget.Gnaf.Grammar) (selectors : List (LexLeanTarget.Gnaf.Selector)) (normal : Nat) (facts : List (Fact)) : Verdict := (match factAt (facts) (normal) with | Option.none => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Option.some found => (match found with | Fact.inadmissible => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.admitted cost size => (match selectorAt (selectors) (normal) with | Option.none => Verdict.incomplete (Obligation.normalOutOfRange) (Established.notCertified) | Option.some selector => (if isNormal (request) (grammar) (selector) then Verdict.incomplete (Obligation.optimalityUnproved) (Established.locallyNormalized (normal)) else Verdict.incomplete (Obligation.systemNotNormal) (Established.correctOnly (normal) (cost) (size)))) | Fact.atLeast _ _ => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.reduces _ => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.unresolved => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified) | Fact.unproven => Verdict.incomplete (Obligation.normalUnmeasured) (Established.notCertified)))

@[expose] public def claimShape (claim : LexLeanTarget.Gnaf.ClaimClass) : Nat := (match claim with | LexLeanTarget.Gnaf.ClaimClass.exact => 0 | LexLeanTarget.Gnaf.ClaimClass.normalForm => 0 | LexLeanTarget.Gnaf.ClaimClass.canonical => 0 | LexLeanTarget.Gnaf.ClaimClass.representationMinimal => 0 | LexLeanTarget.Gnaf.ClaimClass.comparisonTheorem => 0 | LexLeanTarget.Gnaf.ClaimClass.profileDefinedComparison _ _ _ => 0 | LexLeanTarget.Gnaf.ClaimClass.inputTotal => 0 | LexLeanTarget.Gnaf.ClaimClass.globalOptimal => 1 | LexLeanTarget.Gnaf.ClaimClass.argminComplete => 2 | LexLeanTarget.Gnaf.ClaimClass.paretoOptimal => 3 | LexLeanTarget.Gnaf.ClaimClass.frontierComplete => 4 | LexLeanTarget.Gnaf.ClaimClass.pointwiseEnvelopeComplete => 0 | LexLeanTarget.Gnaf.ClaimClass.queryFamilyAnswerComplete => 0 | LexLeanTarget.Gnaf.ClaimClass.useCaseGlobalOptimal => 0 | LexLeanTarget.Gnaf.ClaimClass.workloadArgminComplete => 0 | LexLeanTarget.Gnaf.ClaimClass.workloadParetoOptimal => 0 | LexLeanTarget.Gnaf.ClaimClass.workloadFrontierComplete => 0 | LexLeanTarget.Gnaf.ClaimClass.familyOptimal => 0 | LexLeanTarget.Gnaf.ClaimClass.competitiveBound => 0 | LexLeanTarget.Gnaf.ClaimClass.competitiveOptimal => 0 | LexLeanTarget.Gnaf.ClaimClass.asymptoticBound => 0 | LexLeanTarget.Gnaf.ClaimClass.asymptoticOptimal => 0 | LexLeanTarget.Gnaf.ClaimClass.useCaseClassComplete => 0 | LexLeanTarget.Gnaf.ClaimClass.useCaseClassAnswerComplete => 0 | LexLeanTarget.Gnaf.ClaimClass.maintainedUseCaseClass => 0 | LexLeanTarget.Gnaf.ClaimClass.restrictedUniverseOptimal => 0 | LexLeanTarget.Gnaf.ClaimClass.revisionPreserved => 0 | LexLeanTarget.Gnaf.ClaimClass.bestKnown => 0 | LexLeanTarget.Gnaf.ClaimClass.measuredBestAmongTested => 0 | LexLeanTarget.Gnaf.ClaimClass.heuristicSelected => 0 | LexLeanTarget.Gnaf.ClaimClass.instanceOptimal _ _ => 0)

@[expose] public def sameObjective (left : LexLeanTarget.Gnaf.Objective) (right : LexLeanTarget.Gnaf.Objective) : Bool := (match left with | LexLeanTarget.Gnaf.Objective.scalar => (match right with | LexLeanTarget.Gnaf.Objective.scalar => true | LexLeanTarget.Gnaf.Objective.vector => false) | LexLeanTarget.Gnaf.Objective.vector => (match right with | LexLeanTarget.Gnaf.Objective.scalar => false | LexLeanTarget.Gnaf.Objective.vector => true))

@[expose] public def sameScope (left : LexLeanTarget.Gnaf.Scope) (right : LexLeanTarget.Gnaf.Scope) : Bool := (match left with | LexLeanTarget.Gnaf.Scope.grammarUniverse => (match right with | LexLeanTarget.Gnaf.Scope.grammarUniverse => true | LexLeanTarget.Gnaf.Scope.calculusPrograms => false | LexLeanTarget.Gnaf.Scope.rustPrograms => false) | LexLeanTarget.Gnaf.Scope.calculusPrograms => (match right with | LexLeanTarget.Gnaf.Scope.grammarUniverse => false | LexLeanTarget.Gnaf.Scope.calculusPrograms => true | LexLeanTarget.Gnaf.Scope.rustPrograms => false) | LexLeanTarget.Gnaf.Scope.rustPrograms => (match right with | LexLeanTarget.Gnaf.Scope.grammarUniverse => false | LexLeanTarget.Gnaf.Scope.calculusPrograms => false | LexLeanTarget.Gnaf.Scope.rustPrograms => true))

@[expose] public def check (request : LexLeanTarget.Gnaf.Request) (certificate : Certificate) : Verdict := (if (!(LexLeanRuntime.equal ((certificate).verifier) ("lexlean/gnaf-certification/1") : Bool)) then Verdict.refused (Refusal.verifierMismatch) else (match LexLeanTarget.Gnaf.validate (request) with | Option.none => (match (request).carrier with | LexLeanTarget.Gnaf.Carrier.grammar grammar => (if (!(LexLeanRuntime.equal ((certificate).«universe») ((request).«universe») : Bool)) then Verdict.refused (Refusal.universeMismatch) else (if (!sameObjective ((certificate).objective) ((request).objective)) then Verdict.refused (Refusal.objectiveMismatch) else (if (!((!(Nat.beq (claimShape ((certificate).claim)) (0))) && (Nat.beq (claimShape ((certificate).claim)) (claimShape ((request).claim))))) then Verdict.refused (Refusal.claimMismatch) else (if (!sameScope ((certificate).scope) ((request).scope)) then Verdict.refused (Refusal.scopeMismatch) else (let selectors : List (LexLeanTarget.Gnaf.Selector) := LexLeanTarget.Gnaf.expand (grammar); (let count : Nat := (LexLeanRuntime.length (selectors) : Nat); (let given : Nat := (LexLeanRuntime.length ((certificate).premises) : Nat); (if (Nat.blt (given) (count)) then Verdict.refused (Refusal.missingPremise (given)) else (if (Nat.blt (count) (given)) then Verdict.refused (Refusal.extraPremise (count)) else (let facts : List (Fact) := factsOf (request) (grammar) (selectors) (selectors) ((certificate).premises); (let shape : Nat := claimShape ((certificate).claim); (match (certificate).route with | Route.lowerBound bound attainers => (if (Nat.blt (shape) (3)) then lowerVerdict ((Nat.beq (shape) (2))) (bound) (attainers) (facts) else Verdict.refused (Refusal.routeIncompatible)) | Route.frontierCover members => (if (Nat.ble (3) (shape)) then frontierVerdict (members) (facts) else Verdict.refused (Refusal.routeIncompatible)) | Route.reduction => reductionVerdict (shape) (facts) | Route.representation normal bridge => (if (Nat.beq (shape) (1)) then representationVerdict (normal) (bridge) (facts) (sizesOf (grammar) (selectors)) else Verdict.refused (Refusal.routeIncompatible)) | Route.normality normal => normalityVerdict (request) (grammar) (selectors) (normal) (facts))))))))))))) | LexLeanTarget.Gnaf.Carrier.internalPlans => Verdict.refused (Refusal.requestRejected (LexLeanTarget.Gnaf.Rejection.internalPlanUniverse)) | LexLeanTarget.Gnaf.Carrier.optimizerOutput => Verdict.refused (Refusal.requestRejected (LexLeanTarget.Gnaf.Rejection.optimizerDefinedUniverse)) | LexLeanTarget.Gnaf.Carrier.discovered _ => Verdict.refused (Refusal.requestRejected (LexLeanTarget.Gnaf.Rejection.discoveredUniverse)) | LexLeanTarget.Gnaf.Carrier.cached => Verdict.refused (Refusal.requestRejected (LexLeanTarget.Gnaf.Rejection.cachedUniverse))) | Option.some rejection => Verdict.refused (Refusal.requestRejected (rejection))))

end LexLeanTarget.Certification

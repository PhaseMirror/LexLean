open Lean Compiler LCNF

namespace LexLeanExtract

meta def escape (text : String) : String :=
  text.foldl (fun out c =>
    if c == '"' then out ++ "\\\""
    else if c == '\\' then out ++ "\\\\"
    else if c.toNat < 0x20 then out ++ "\\u" ++ String.ofList ((Nat.toDigits 16 (c.toNat + 0x10000)).drop 1)
    else out.push c) ""

meta def str (text : String) : String := "\"" ++ escape text ++ "\""

meta def name (n : Name) : String := str n.toString

meta def arr (items : List String) : String := "[" ++ String.intercalate "," items ++ "]"

meta def obj (fields : List (String × String)) : String :=
  "{" ++ String.intercalate "," (fields.map fun (key, value) => str key ++ ":" ++ value) ++ "}"

meta def bool (value : Bool) : String := if value then "true" else "false"

meta def fvar (id : FVarId) : String := name id.name

meta partial def type (e : Expr) : String :=
  match e with
  | .const n _ =>
    if n == ``lcErased then obj [("kind", str "erased")]
    else if n == ``lcAny then obj [("kind", str "any")]
    else obj [("kind", str "const"), ("name", name n)]
  | .app .. =>
    obj [("kind", str "app"), ("head", type e.getAppFn), ("arguments", arr (e.getAppArgs.toList.map type))]
  | .forallE _ domain body _ =>
    obj [("kind", str "arrow"), ("domain", type domain), ("codomain", type body)]
  | .fvar id => obj [("kind", str "fvar"), ("id", fvar id)]
  | .bvar index => obj [("kind", str "bvar"), ("index", toString index)]
  | .sort _ => obj [("kind", str "sort")]
  | _ => obj [("kind", str "unsupported"), ("expression", str (toString e))]

meta def arg (a : Arg .pure) : String :=
  match a with
  | .erased => obj [("kind", str "erased")]
  | .fvar id => obj [("kind", str "fvar"), ("id", fvar id)]
  | .type e => obj [("kind", str "type"), ("type", type e)]

meta def literal (value : LitValue) : String :=
  match value with
  | .nat v => obj [("kind", str "nat"), ("value", str (toString v))]
  | .str v => obj [("kind", str "string"), ("value", str v)]
  | .uint8 v => obj [("kind", str "uint8"), ("value", str (toString v.toNat))]
  | .uint16 v => obj [("kind", str "uint16"), ("value", str (toString v.toNat))]
  | .uint32 v => obj [("kind", str "uint32"), ("value", str (toString v.toNat))]
  | .uint64 v => obj [("kind", str "uint64"), ("value", str (toString v.toNat))]
  | .usize v => obj [("kind", str "usize"), ("value", str (toString v.toNat))]

meta def letValue (v : LetValue .pure) : String :=
  match v with
  | .lit value => obj [("kind", str "literal"), ("literal", literal value)]
  | .erased => obj [("kind", str "erased")]
  | .proj typeName index struct => obj [("kind", str "projection"), ("type_name", name typeName), ("index", toString index), ("value", fvar struct)]
  | .const declName _ args => obj [("kind", str "const"), ("name", name declName), ("arguments", arr (args.toList.map arg))]
  | .fvar id args => obj [("kind", str "apply"), ("function", fvar id), ("arguments", arr (args.toList.map arg))]

meta def param (p : Param .pure) : String :=
  obj [("id", fvar p.fvarId), ("type", type p.type), ("borrow", bool p.borrow)]

mutual
meta partial def code (c : Code .pure) : String :=
  match c with
  | .let decl k => obj [("kind", str "let"), ("id", fvar decl.fvarId), ("type", type decl.type), ("value", letValue decl.value), ("body", code k)]
  | .fun decl k => obj [("kind", str "fun"), ("declaration", funDecl decl), ("body", code k)]
  | .jp decl k => obj [("kind", str "join"), ("declaration", funDecl decl), ("body", code k)]
  | .jmp id args => obj [("kind", str "jump"), ("target", fvar id), ("arguments", arr (args.toList.map arg))]
  | .cases cs => obj [("kind", str "cases"), ("type_name", name cs.typeName), ("result_type", type cs.resultType), ("discriminant", fvar cs.discr), ("alternatives", arr (cs.alts.toList.map alt))]
  | .return id => obj [("kind", str "return"), ("id", fvar id)]
  | .unreach t => obj [("kind", str "unreachable"), ("type", type t)]

meta partial def funDecl (d : FunDecl .pure) : String :=
  obj [("id", fvar d.fvarId), ("parameters", arr (d.params.toList.map param)), ("type", type d.type), ("value", code d.value)]

meta partial def alt (a : Alt .pure) : String :=
  match a with
  | .alt ctorName params k => obj [("kind", str "constructor"), ("constructor", name ctorName), ("parameters", arr (params.toList.map param)), ("code", code k)]
  | .default k => obj [("kind", str "default"), ("code", code k)]
end

meta def kind (info : ConstantInfo) : String :=
  match info with
  | .defnInfo value =>
    match value.safety with
    | .safe => "definition"
    | .unsafe => "unsafe-definition"
    | .partial => "partial-definition"
  | .opaqueInfo _ => "opaque"
  | .thmInfo _ => "theorem"
  | .axiomInfo _ => "axiom"
  | .inductInfo _ => "inductive"
  | .ctorInfo _ => "constructor"
  | .recInfo _ => "recursor"
  | .quotInfo _ => "quotient"

meta partial def typeConstants (e : Expr) (out : NameSet) : NameSet :=
  match e with
  | .const n _ => if n == ``lcErased || n == ``lcAny then out else out.insert n
  | .app f a => typeConstants a (typeConstants f out)
  | .forallE _ d b _ => typeConstants b (typeConstants d out)
  | _ => out

meta def argConstants (a : Arg .pure) (out : NameSet) : NameSet :=
  match a with
  | .type e => typeConstants e out
  | _ => out

meta def paramConstants (ps : Array (Param .pure)) (out : NameSet) : NameSet :=
  ps.foldl (fun acc p => typeConstants p.type acc) out

mutual
meta partial def codeConstants (c : Code .pure) (out : NameSet) : NameSet :=
  match c with
  | .let decl k =>
    let out := typeConstants decl.type out
    let out := match decl.value with
      | .const n _ args => args.foldl (fun acc a => argConstants a acc) (out.insert n)
      | .fvar _ args => args.foldl (fun acc a => argConstants a acc) out
      | .proj typeName _ _ => out.insert typeName
      | _ => out
    codeConstants k out
  | .fun decl k => codeConstants k (funConstants decl out)
  | .jp decl k => codeConstants k (funConstants decl out)
  | .jmp _ args => args.foldl (fun acc a => argConstants a acc) out
  | .cases cs =>
    cs.alts.foldl (fun acc a => match a with
      | .alt ctorName ps k => codeConstants k (paramConstants ps (acc.insert ctorName))
      | .default k => codeConstants k acc) (typeConstants cs.resultType (out.insert cs.typeName))
  | .return _ => out
  | .unreach t => typeConstants t out

meta partial def funConstants (d : FunDecl .pure) (out : NameSet) : NameSet :=
  codeConstants d.value (typeConstants d.type (paramConstants d.params out))
end

meta def moduleOf (env : Environment) (n : Name) : Name :=
  match env.getModuleIdxFor? n with
  | some index => env.header.moduleNames[index.toNat]!
  | none => Name.anonymous

meta def owned (env : Environment) (modules runtime : Array Name) (n : Name) : Bool :=
  modules.contains (moduleOf env n) && !runtime.any (fun namespacePrefix => namespacePrefix.isPrefixOf n)

meta def sorted (names : NameSet) : List Name :=
  (names.toList.toArray.qsort Name.lt).toList

meta def walk (roots modules runtime : Array Name) : CoreM (Array String × Array Name × NameSet × NameSet) := do
  let env ← getEnv
  let mut declarations : Array String := #[]
  let mut names : Array Name := #[]
  let mut referenced : NameSet := {}
  let mut erased : NameSet := {}
  let mut done : NameSet := {}
  let mut pending : Array Name := roots
  let mut cursor := 0
  while cursor < pending.size do
    let n := pending[cursor]!
    cursor := cursor + 1
    if done.contains n then continue
    done := done.insert n
    let some info := env.find? n
      | throwError "lexlean-extract: unknown constant `{n}`"
    let k := kind info
    let computable := !isNoncomputable env n
    let generates ← shouldGenerateCode n
    if k != "definition" || !computable || !generates then
      declarations := declarations.push (obj [("name", name n), ("kind", str k), ("computable", bool computable), ("generates_code", bool generates)])
      continue
    names := names.push n
    let decl ← CompilerM.run (toDecl n)
    let mut scan : Array Name := (info.value? (allowOpaque := true)).map Expr.getUsedConstants |>.getD #[]
    let mut scanned : NameSet := {}
    let mut position := 0
    while position < scan.size do
      let used := scan[position]!
      position := position + 1
      if scanned.contains used then continue
      scanned := scanned.insert used
      if let some usedInfo := env.find? used then
        if owned env modules runtime used then
          if usedInfo.isTheorem then
            erased := erased.insert used
          if used.isInternal then
            scan := scan ++ ((usedInfo.value? (allowOpaque := true)).map Expr.getUsedConstants |>.getD #[])
    let signature : NameSet := typeConstants decl.type (paramConstants decl.params {})
    let (value, uses) := match decl.value with
      | .code c => (obj [("kind", str "code"), ("code", code c)], codeConstants c signature)
      | .extern _ => (obj [("kind", str "extern")], signature)
    declarations := declarations.push (obj [
      ("name", name n), ("kind", str k), ("computable", "true"), ("generates_code", "true"),
      ("safe", bool decl.safe), ("recursive", bool decl.recursive),
      ("level_parameters", arr (decl.levelParams.map name)),
      ("type", type decl.type), ("parameters", arr (decl.params.toList.map param)), ("value", value)])
    for r in sorted uses do
      match env.find? r with
      | some (.ctorInfo ctor) =>
        referenced := referenced.insert ctor.induct
        referenced := referenced.insert r
      | some (.inductInfo _) => referenced := referenced.insert r
      | some _ =>
        if owned env modules runtime r then pending := pending.push r else referenced := referenced.insert r
      | none => throwError "lexlean-extract: unresolved constant `{r}`"
  return (declarations, names, referenced, erased)

meta def run (roots modules runtime : Array Name) : CoreM String := do
  let env ← getEnv
  let (declarations, _, referenced, erased) ← walk roots modules runtime
  let mut closures : Array String := #[]
  for root in roots do
    let (_, names, _, rootErased) ← walk #[root] modules runtime
    let (rootInternal, rootNamed) := (sorted rootErased).partition Name.isInternal
    closures := closures.push (obj [
      ("root", name root),
      ("declarations", arr ((names.qsort Name.lt).toList.map name)),
      ("erased", arr (rootNamed.map name)),
      ("internal_erased", arr (rootInternal.map name))])
  let mut inductiveRows : Array String := #[]
  let mut externalRows : Array String := #[]
  for r in sorted referenced do
    let some info := env.find? r | throwError "lexlean-extract: unresolved constant `{r}`"
    match info with
    | .inductInfo value =>
      if owned env modules runtime r then
        let mut ctors : Array String := #[]
        for (c, index) in value.ctors.zipIdx do
          let some (.ctorInfo ctor) := env.find? c | throwError "lexlean-extract: `{c}` is not a constructor"
          let ctorType ← Meta.MetaM.run' (toLCNFType ctor.type)
          ctors := ctors.push (obj [("name", name c), ("index", toString index), ("parameters", toString ctor.numParams), ("fields", toString ctor.numFields), ("type", type ctorType)])
        inductiveRows := inductiveRows.push (obj [("name", name r), ("parameters", toString value.numParams), ("indices", toString value.numIndices), ("recursive", bool value.isRec), ("constructors", arr ctors.toList)])
      else
        externalRows := externalRows.push (obj [("name", name r), ("kind", str (kind info)), ("module", name (moduleOf env r)), ("computable", "true"), ("generates_code", "false")])
    | .ctorInfo ctor =>
      if !owned env modules runtime ctor.induct then
        externalRows := externalRows.push (obj [("name", name r), ("kind", str (kind info)), ("module", name (moduleOf env r)), ("computable", "true"), ("generates_code", "false")])
    | _ =>
      let generates ← shouldGenerateCode r
      externalRows := externalRows.push (obj [("name", name r), ("kind", str (kind info)), ("module", name (moduleOf env r)), ("computable", bool (!isNoncomputable env r)), ("generates_code", bool generates)])
  let (internal, named) := (sorted erased).partition Name.isInternal
  return obj [
    ("spec", str "lexlean/lcnf-extraction/1"),
    ("lean", obj [("version", str Lean.versionString), ("githash", str Lean.githash)]),
    ("roots", arr (roots.toList.map name)),
    ("closures", arr closures.toList),
    ("declarations", arr declarations.toList),
    ("inductives", arr inductiveRows.toList),
    ("externals", arr externalRows.toList),
    ("erased", arr (named.map name)),
    ("internal_erased", arr (internal.map name))]

meta def main (roots modules runtime : Array Name) : Lean.Elab.Command.CommandElabM Unit :=
  Lean.Elab.Command.liftCoreM do
    IO.println (← run roots modules runtime)

end LexLeanExtract

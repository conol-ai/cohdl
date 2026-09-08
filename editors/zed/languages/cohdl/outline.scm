(declaration
  kind: [
    "device" "trait" "part" "fn"
    "design" "subdesign" "footprint" "pad"
  ] @context
  name: (identifier) @name) @item

(impl_declaration
  "impl" @context
  trait: (path) @name) @item

(inst_statement
  "inst" @context
  name: (identifier) @name) @item

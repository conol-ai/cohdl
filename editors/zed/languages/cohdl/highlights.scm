; CoHDL highlighting for Zed. Scope coverage mirrors the VS Code TextMate
; grammar (RFC-019/DR-025) extended through RFC-032: structural keywords are
; grammar tokens; statement keywords, RFC-002 roles, and RFC-001 unit types
; are contextual identifier matches — the same single-class discipline.

(comment) @comment
(string) @string
(unit_literal) @constant
(number) @number
(wildcard) @constant

[
  "pub" "device" "trait" "part" "fn" "design" "subdesign"
  "footprint" "pad" "use" "impl" "for" "inst" "net"
] @keyword

(declaration name: (identifier) @type)
(use_declaration path: (path (identifier) @namespace))
(impl_declaration trait: (path (identifier) @type))
(impl_declaration device: (path (identifier) @type))
(inst_statement name: (identifier) @variable)
(net_statement name: (identifier) @variable)

(attribute (identifier) @attribute)
["#["] @attribute

; Body / statement keywords (contextual identifiers).
((identifier) @keyword
 (#any-of? @keyword
  "pins" "spec" "nc" "ports" "variants" "layout"
  "required" "optional" "primary" "alt"
  "courtyard" "silkscreen" "silkscreen_ref"
  "net_class" "diff_pair" "length_match" "board_outline"
  "place" "at" "rotate" "side" "top" "bottom"
  "mount_hole" "diameter" "shape" "size" "drill" "plating" "near"
  "designator_prefix" "mfr" "mpn" "tolerance"
  "line" "circle" "arc" "polygon"
  "pin_1_marker" "polarity_marker" "cathode_pin"
  "dot" "triangle" "band" "arrow"))

; RFC-002 pin connection-obligation roles.
((identifier) @constant
 (#any-of? @constant
  "passive" "power_in" "power_out" "output" "input" "bidirectional" "gnd"))

; RFC-001 unit type names + Pin (RFC-006) + pin (RFC-002 trait-pin type).
((identifier) @type
 (#any-of? @type
  "Voltage" "Capacitance" "Resistance" "Current" "Frequency" "Time"
  "Inductance" "Power" "Temperature" "Tolerance" "Length" "Pin" "pin"))

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
[":" "," "." ";"] @punctuation.delimiter
["::" "<" ">" "=" "..="] @operator

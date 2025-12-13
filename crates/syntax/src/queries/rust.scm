; Keywords
[
  "as"
  "async"
  "await"
  "break"
  "const"
  "continue"
  "dyn"
  "else"
  "enum"
  "extern"
  "fn"
  "for"
  "if"
  "impl"
  "in"
  "let"
  "loop"
  "match"
  "mod"
  "move"
  "pub"
  "ref"
  "return"
  "static"
  "struct"
  "trait"
  "type"
  "unsafe"
  "use"
  "where"
  "while"
  "mut"
  "crate"
  "self"
  "super"
] @keyword

; Function definitions
(function_item
  name: (identifier) @function)

(function_signature_item
  name: (identifier) @function)

; Function calls
(call_expression
  function: (identifier) @function)

(call_expression
  function: (field_expression
    field: (field_identifier) @function))

; Types
(type_identifier) @type
(primitive_type) @type

; Strings
(string_literal) @string
(raw_string_literal) @string
(char_literal) @string

; Comments
(line_comment) @comment
(block_comment) @comment

; Numbers
(integer_literal) @number
(float_literal) @number

; Constants
(boolean_literal) @constant
((identifier) @constant
 (#match? @constant "^[A-Z][A-Z_0-9]*$"))

; Properties
(field_identifier) @property

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "%"
  "="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "!"
  "&"
  "|"
  "^"
  "<<"
  ">>"
  "+="
  "-="
  "*="
  "/="
  "%="
  "&="
  "|="
  "^="
  "<<="
  ">>="
  "->"
  "=>"
  ".."
  "..="
] @operator

; Punctuation
[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
  ","
  ";"
  ":"
  "::"
  "."
] @punctuation

; Keywords
[
  "async"
  "await"
  "break"
  "case"
  "catch"
  "class"
  "const"
  "continue"
  "debugger"
  "default"
  "delete"
  "do"
  "else"
  "export"
  "extends"
  "finally"
  "for"
  "from"
  "function"
  "if"
  "import"
  "in"
  "instanceof"
  "let"
  "new"
  "of"
  "return"
  "static"
  "switch"
  "throw"
  "try"
  "typeof"
  "var"
  "void"
  "while"
  "with"
  "yield"
] @keyword

; Function definitions
(function_declaration
  name: (identifier) @function)

(method_definition
  name: (property_identifier) @function)

(function_expression
  name: (identifier) @function)

; Function calls
(call_expression
  function: (identifier) @function)

(call_expression
  function: (member_expression
    property: (property_identifier) @function))

; Types (TypeScript-like)
(type_identifier) @type

; Strings
(string) @string
(template_string) @string

; Comments
(comment) @comment

; Numbers
(number) @number

; Constants
[
  "true"
  "false"
  "null"
  "undefined"
] @constant

; Properties
(property_identifier) @property

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "%"
  "="
  "=="
  "==="
  "!="
  "!=="
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
  "~"
  "<<"
  ">>"
  ">>>"
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
  ">>>="
  "=>"
  "..."
  "??"
  "?."
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
  "."
  "?"
] @punctuation

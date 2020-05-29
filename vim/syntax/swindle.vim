if exists("b:current_syntax")
    finish
endif

let b:current_syntax = "swindle"

syntax keyword swindleKeyword and or not
syntax keyword swindleKeyword if else elif
syntax keyword swindleKeyword while break continue for loop
syntax keyword swindleKeyword int string bool unit fn
highlight link swindleKeyword Keyword

syntax keyword swindleBoolean true false
highlight link swindleBoolean Boolean

syntax match swindleGlobal "\v\@[_a-zA-Z0-9]+"
highlight link swindleGlobal Operator

syntax match swindleNumber "\v\d+"
highlight link swindleNumber Number

syntax region swindleString start=/\v"/ skip=/\v\\./ end=/\v"/
highlight link swindleString String

syntax region swindleBlock start="{" end="}" transparent fold
syntax region swindleParen start="(" end=")" transparent fold

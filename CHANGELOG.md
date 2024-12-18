1.6.1
=====

* Allow " characters in multiline hints

1.6.0
=====

* Switch from # to ; for comments. This allows HTML colors to be used without quotes.
* Rename `Token::MultilineIndicator` to `Token::MultilineHint`

1.5.0
=====

* Switch from escape codes to strings. Although the escape codes are likely used less often, it is quite inconvenient to represent a key with multiple = signs using them.

1.4.0
=====

* Allow values to be omitted (necessary now comments no longer participate in indentation).
* Remove `"{}` for empty.

1.3.0
=====

* Comments no-longer participate in indentation.
* Removed `"\`.

1.2.0
=====

* Removed the requirement for a space before the `=` sign.
* Replaced `"@` with `"{}` to represent an empty string. Clarified that it can only appear on its own.
* Limited the length of `"{X}` escapes to 6 hex bytes.
* Clarified the syntax of multiline syntax indicators.

1.1.0
=====

* Added `Token::line_number()`
* Added `Token::name()`
* Fixed a bug parsing multiline values

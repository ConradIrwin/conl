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

# Command Reference

This page contains the complete reference for all available printer commands in the escpos2mqtt DSL.
This document describes the Domain Specific Language (DSL) used to send printing commands to ESC/POS-compatible printers.
The documentation below is automatically generated from the parser implementation.

## Overview
The DSL consists of commands that are executed sequentially. Each command must be on its own line.
Empty lines are ignored. String arguments must be enclosed in double quotes.

<!-- cmdrun cargo run --bin generate_docs markdown -->

## Complete Example

```
justify center
bold true
size 2,2
writeln "RECEIPT"
reset_size
bold false
feed 1
justify left
writeln "Item 1          $10.00"
writeln "Item 2          $15.00"
underline single
writeln "Total:          $25.00"
underline none
feed 2
justify center
qr_code "https://example.com/receipt/12345"
feed 2
cut
```

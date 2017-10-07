# SQL Server Resolution Protocol(SSRP) library for Rust

this is the crate which gets SQL Server instance information using SSRP.

## Preparation

1. add `ssrp = { git = "https://github.com/itn3000/ms-ssrp-rs" }` to your Cargo dependency
2. boot sqlbrowser in your SQL Server machine
3. open firewall(usually, 1434/udp)

## Example code

see [examples folder](./examples/)

## About SSRP

see [Microsoft's Document](https://msdn.microsoft.com/en-us/library/cc219703.aspx)
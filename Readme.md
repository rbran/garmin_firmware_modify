This project is a simple Proof-of-concept that compile a payload and inject it
into a `edge130apac` firmware.

There are two sup-projects:

## firmware_inject

Using the lib [gcd-rs](https://github.com/rbran/gcd-rs) it is able to modify the
original firmware file, injecting a payload that will be executed on boot.

## firmware_payload

Payload (no_std) that will be executed on the boot. There are two available
payloads:

### mem_dump

Dumps the content of the ram into a file on the flash memory.


### port_search

flip all the GPIO ports in a predictable patter, so it can be identified using
a logic analyzer.

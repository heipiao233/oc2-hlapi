# OC2-HLAPI

Rust bindings for the [OpenComputers II](https://www.curseforge.com/minecraft/mc-mods/oc2r)
Minecraft mod's HLAPI.

## Credits and inspiration

Much of my understanding of OC2's API came from studying the source code of the mod itself and the
[`miku-rpc`](https://crates.io/crates/miku-rpc) project. Thank you to the creators of both. I created
this library because I desired stronger typing than what either `miku-rpc` or the lua interface of
OC2 provided.

This library doesn't offer lua bindings for its functionality, so if that's something you're looking
for, I would highly recommend using `miku-rpc` instead.

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a> or
<a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
oc2-hlapi by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
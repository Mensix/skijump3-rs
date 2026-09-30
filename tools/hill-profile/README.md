# hill-profile

Generation-only tool for runtime-compatible SJ3 custom hills. The only command is:

```text
cargo run -p hill-profile -- generate profile.toml output-dir
```

The command directly writes the generated assets to the output directory. It does
not depend on the game, run an AI probe, or prompt for confirmation.

The TOML profile contains `id`, optional `name` and metadata, `width`, `height`,
`tip_x`, `tip_drop`, and `[[points]]` entries with `section = "before"` or
`section = "after"`, `x`, `y`, and an optional `slope`. Both sections must contain
`tip_x`; their y values define the explicit vertical takeoff discontinuity and
must match `tip_drop`. Points are validated for dimensions, finite values,
strictly increasing x, and monotone y.

The curve is generated using piecewise monotone PCHIP cubic Hermite segments
with explicit vertical takeoff discontinuity, not a single global curve. The
front terrain is rasterized into an alpha-compatible PNG and the back is
procedurally generated. Output is written to:

```text
output-dir/hills/generated/HILL<ID>/{front,back}.png
output-dir/custom_hills/<ID>.toml
```

The custom hill metadata uses the game-compatible profile and metadata
checksums. The profile dimensions are normally `1024x512`, but other positive
dimensions accepted by the runtime image path can be used.

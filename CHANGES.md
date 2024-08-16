### v0.8.0
breaking changes:
- inputs to `shift_reg()` are swapped (signal | trigger) 2fc246c

new:
- `snh()` sample-and-hold node f925233
- `s()` decimation node (like kr() but preserves time) ef1e1a7
- `worm` wireless many-to-many number connections 16ed66e

fixes:
- avoid panic with the `st` command 9dca14e
- avoid panic with `swap()` 3ac8312

### v0.7.0
fixes:
- avoid possible crash when abusing the arity of `swap()` 77baf38
- avoid deadlock when input sending/receiving fails a7700ac
- limit `render`'s output length eaa8842

new:
- make `swap()` work without the need to specify arity in op string ca74b8a (not a breaking change)
- make `kr()` work with arbitrary arity 181eaed a2d8e80 5b368a1 89ba1ba
- buffer nodes to get samples from the audio graph and use outside 42048bc 945424d
- `:reset_cam` command dd0e9b8

### v0.6.3
fixes:
- avoid possible panic when selecting entities ebcaf5c
- remove the Save component from saved scenes f8d3fb2 (not breaking backwards compatibility)

known issues:
- i forgot to update the version number in cargo.toml so the version command will print 0.6.2 (but correct hash)

### v0.6.2
fixes:
- avoid panic when duplicating 17fa913
- optimizations

### v0.6.1
breaking changes / fixes:
- added an offset parameter to fft nodes to allow them to perform correct overlap, and fixed the examples 984b843

### v0.6.0
breaking changes:
- existing scenes must adapt these changes to run in 0.6.0 (see commit message) d20a5e6
- having multiple out() nodes was allowed, and their inputs were mixed together. now the last one to have an open white hole will set the output to that node, so do your mixing first eb6ac41
- `count` is removed. use `feedback()` instead e7bbb90b6d3
- `repeat` only works on targets now. its num determines the repetitions. use repeated branch/stack ops to repeat arrays 884e2b7a
- `render` used to work with nodes of any output arity, now the node must have 1 output
- allpole_delay() is merged into allpole() 04e8d96

fixes:
- #86 and #182 are fixed by the fundsp 0.18 update 079ac86
- avoid overflows with `shl()` and `shr()` 17f27f4
- clipboard now works in wasm 1185a2d
- info texts are shown by default. use `vt` to toggle them, and `vT` to toggle ids. `II`,`IC`,`ID` commands are removed

new:
- distro can set order of targets cfbbe3f
- `swap()` a node swapping node 3d96d6b f3bd754
- `:reset_bloom` command
- `rfft()` and `ifft()` 81ce3cf a5771d3
- `chan()` 294788a
- `samp_delay()` 2eda517

known issues:
- the fft examples suck! i'm pretty sure that's just because i'm terrible at spectral processing. but the nodes themselves should be fine.

POWER!

### v0.5.0
new:
- `np` command will display number of nodes contained in the selected node 1eedb9a
- selection is now rectangular 16af807
- `ah` `ao` `ai` commands to display info about audio devices
- `:od`/`:id` commands to select output/input devices, sample rate, and buffer size
- `sr()` op to set the sampling rate of an audio graph
- jack support
- `in()`/`adc()` node for audio input #135
- `command` op to control the command line a5466eb

fixes:
- connective ops will limit themselves to 500 nodes, this avoids the need for order checking when connecting audio nodes and fixes other related issues (#158 and #142)
  - this limit can be set using the `:nl` command

breaking changes:
- clock is removed be597d9 use a `ramp()` `>>` `<(0.5)` for that same behavior
- `selection_color` and `connecting_line_color` are now removed and `indicator_color` replaces both 05dd0e4

### v0.4.0
fixes:
- `seq()` will only allow one playing event for any node (this fixes the jump in frequency when a lot of events for the same node are triggered at the same time, which was caused by the node being ticked more than once in the same sample) 77e138a
- `np` command only printed info of one circle even if more were selected 4595b90
- inputs to nodes connected with `*` or `+` were not stacked correctly 6c8d95a

new potatoes:
- deselect-all `sA` and select-targets `st` commands 4508b0a
- when a scene is pasted or loaded those entities are selected 705a19e
- `reset_v()` node for resetting a given node at a variable interval 3d9a064
- `ramp()` is now audio-rate 2165d98
- `pdhalf_bi()` and `pdhalf_uni()` nodes 28a74d9
- `push_num` op b984325
- `}` and `{` shortcuts for stepping link types 1cf7f5e
- set x and y radii independently 96b1399

### v0.3.0
new junk:
- deg, rad, and recip nodes a5804e2
- select_target op fc0c68d
- connection_width op b55be2e
- text_size op 786b1dc
- osc, osc_s, and osc_r ops #156
- trig_reset node e57295b

crashes:
- duplicating while something is sending a constant OrderChange event b51ed80 (#155)

### v0.2.0
breaking changes:
- render b9f32368bb211009c

new junk:
- pressed op 40323b7bce3f1486
- atan2 and hypot nodes a0db0b9
- mirror node eb7e9ac 61e26b3
- cartesian and polar nodes 7845e7ec02540

crashes:
- duplication that leaves inexistent id's in holes array e1fe1895a4c1
- disallow duplicating holes separately 3a55f86 (and avoid possible crash when applying the command buffer)

### v0.1.0

the circle it is drawn

the spell it is cast

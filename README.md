# Quartz

⚠️ under construction ⚠️

![Screenshot_2024-02-21_19-39-17](https://github.com/tomara-x/quartz/assets/86204514/0102a8ff-5c56-41f9-be1a-446d4e1a34d4)
![Screenshot_2024-03-31_04-11-18](https://github.com/tomara-x/quartz/assets/86204514/5b43d686-4e55-4025-b342-502cafaac534)
![Screenshot_2024-03-31_04-12-09](https://github.com/tomara-x/quartz/assets/86204514/8dca28da-788e-49ca-81aa-1ec9c4dad9fd)

```
"you africans, please listen to me as africans
and you non-africans, listen to me with open mind"
```
## let's play

#### building
- install rust: https://www.rust-lang.org/tools/install
- install bevy dependencies: https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies
- clone quartz
```
git clone https://github.com/tomara-x/quartz.git
```
- build it
```
cd quartz
cargo run --release
```
---
#### modes
when you open quartz, it will be an empty window. there's 3 modes:
- **edit**: (default) interact with entities and execute commands (press `e` or `esc`)
- **draw**: draw new circles (press `d`)
- **connect**: connect circles (press `c`)
    - **target**: target an entity from another (hold `t` in connect mode)
---
#### circle anatomy
first, define your terms:
- circle: an object that you create in draw mode (they're regular polygons)
- hole: a connection object. these always come in (black hole - white hole) pairs
- entity: i'll use that to refer to any object (circle or hole)

in addition to the shared properties that both circles and holes have (position, color, radius, vertices) a circle holds other things:
- a number (just a float)
- an op string: defining what that circle does
     - for example: `sum`, `toggle`, `screenshot`, `lowpass()`
- an array of numbers: for different uses (don't worry it's empty by default, thus allocating no memory)
- an array of [target](#targets) entities that this circle controls in some way (empty by default too)
- an [order](#order) number: defining if/when that circle is processed
- an audio node (defined by the op string) (`dc(0)` by default)
---
#### commands
there are 2 types of commands:
##### return-terminated
(type it then press enter) (you can separate them with `;` to run more than one at once)
- scene saving/loading
    - `:e {file name}` edit (open) a scene file (in the assets path) (no spaces)
    - `:w {file name}` write (save) a scene file (same)

```
:w moth.cute    // saves the current scene as the file "assets/moth.cute" (OVERWRITES)
:e soup.fun     // opens the file "assets/soup.fun" if it's there
```
(dragging and dropping scene files into a window also works)

- set values
    - `:set n [id] {float}` set num value
    - `:set r [id] {float}` set radius
    - `:set x [id] {float}` set x position
    - `:set y [id] {float}` set y position
    - `:set z [id] {float}` set z position (this controls depth. what's in front of what)
    - `:set h [id] {float}` set hue value [0...360]
    - `:set s [id] {float}` set saturation [0...1]
    - `:set l [id] {float}` set lightness [0...1]
    - `:set a [id] {float}` set alpha [0...1]
    - `:set v [id] {float}` set number of vertices (3 or higher)
    - `:set o [id] {float}` set rotation [-pi...pi] (`:set rot` and `:set rotation` also work)
    - `:set op [id] {string}` set op (use shortcut `o`)
    - `:set ord[der] [id] {float}` set order (use `[` and `]` to increment/decrement order)
    - `:set arr[ay] [id] {float float ...}` set the array (space separated)
    - `:set tar[gets] {id id ...}` set targets (if nothing is selected, the first entity gets the rest of the list as its targets)
    - `:tsel {id}` target selected (`:tsel 4v2` sets selected entities as targets of entity 4v2)
    - `:push {float}/{id}` push a number to the array, or an id to the targets array

```
:set n 4v0 42  // will set the num of entity 4v0 to 42
:set n 42      // will set the num values of selected entities to 42
```

- other
    - `:lt {link type}` set [link type](#link-types) of selected holes (use shortcut `l`)
    - `:dv {float}` set default number of vertices of drawn circles
    - `:dc {float} [float] [float] float]` set default color of drawn circles (h s l a)
    - `:ht {id}` toggle open a white hole (by id)
    - `:q` exit (don't combine with other commands using `;`)

##### immediate commands
(these execute when you finish typing them)
- run mode switching
    - `d` go to draw mode
    - `c` go to connect mode
- drag modes (what happens when dragging selected entities, or when arrow keys are pressed)
    - exclusive:
        - `ee` drag nothing (default)
        - `et` drag translation (move entity)
        - `er` drag radius
        - `en` drag number
        - `eh` drag hue
        - `es` drag saturation
        - `el` drag lightness
        - `ea` drag alpha
        - `eo` drag rotation
        - `ev` drag vertices
    - add a drag mode: (to drag multiple properties at the same time)
        - `Et` add translation
        - `Er` add radius
        - `En` add num
        - `Eh` add hue
        - `Es` add saturation
        - `El` add lightness
        - `Ea` add alpha
        - `Eo` add rotation
        - `Ev` add vertices
- white hole
    - `ht` toggle open status
- shortcuts
    - `o` shortcut for `:set op `
    - `l` shortcut for `:lt `
- info texts
    - `II` spawn info texts for selected entities
    - `IC` clear info texts
    - `ID` show/hide entity id in visible info texts
- inspect commands (information about the selected entities)
    - `ii` entity id's
    - `in` number values
    - `ira` radius values
    - `ix` x position
    - `iy` y position
    - `iz` z position
    - `ihu` hue value
    - `is` saturation
    - `il` lightness
    - `ial` alpha
    - `iv` vertices
    - `iro` rotation
    - `ior` order
    - `iop` op
    - `iar` array
    - `iho` holes
    - `it` targets
    - `iL` hole link type
    - `iO` white hole open status
- audio node info
    - `ni` number of inputs
    - `no` number of outputs
    - `np` info about the node
- selection
    - `sa` select all
    - `sc` select all circles
    - `sC` deselect circles
    - `sh` select all holes
    - `sH` deselect holes
    - `sg` select holes of the selected circles
    - `<delete>` delete selected entities
    - `yy` copy selection to clipboard
    - `p` paste copied

note: when drag-selecting, holding `alt` will only select circles (ignores holes), holding `ctrl` will only select holes (ignores circles), and holding `shift` will add to the selection

- visibility
    - `vc` toggle circle visibility
    - `vb` toggle black hole visibility
    - `vw` toggle white hole visibility
    - `va` toggle arrow visibility
    - `vv` show all
- other
    - `<F11>` toggle fullscreen
    - `quartz` shhh!
    - `awa` [awawawa](https://www.youtube.com/watch?v=LLrIGJEz818)
---
#### link types
any connection links 2 circles together in some way. the black hole is taking some data from the source circle, and the white hole is getting that data and feeding it to the sink circle. the link type determines what that data is.
- `n` or `-1` : num
- `r` or `-2` : radius
- `x` or `-3` : x position
- `y` or `-4` : y position
- `z` or `-5` : z position
- `h` or `-6` : hue
- `s` or `-7` : saturation
- `l` or `-8` : lightness
- `a` or `-9` : alpha
- `v` or `-11` : vertices
- `o` or `-12` : rotation
- `A` or `-13` : array
- `T` or `-14` : targets
- positive numbers: used to denote "input number x"
- `0` usually means audio node (or nothing)
a `0 -> 0` connection does nothing (except for specific ops), but when connecting audio nodes, the connection is usually `0 -> x` (x is positive)
---
#### order
every circle has an order (0 or higher). things in order 0 do nothing.

each frame, every circle with a positive order gets processed (does whatever its op defines)
this processing happens.. you guessed it, in *order*

we process things breadth-first. so when a circle is processed, all its inputs must have their data ready (they processed in this frame) to make sure that's the case, their order has to be lower than that of the circle reading their data...

lower order processes first, and the higher the order, the later that circle processes (within the same frame)

unless...

---
#### targets

a circle has an array of "targets" those are just entity id's. so it's like a "pointer" to another circles or hole. think of it as a one-way wireless connection.
some ops make a circle do things to its targets. like [`process`](#process), `del_targets`, `spin_target`, `distro`, `reorder`, `open_target`, `close_target`, `open_nth` (those 3 work on hole targets)

(they allow some things that aren't easy through normal processing. since circles read their input when they process, while targets are written to when the controller circle is processed instead)

---
#### process
`process` is an op that will process its targets in the order they appear in that targets array. it doesn't care about those targets' order. even if they're at order 0 (it's preferable they are at 0 so you don't cause unexpected things)

so for every frame a circle with a `process` op is processed, it processes all of its targets in order.

combining that with the "reorder" and "rise" ops can allow you to perform things like that in a triggered way.

combining that with the ability to store any number of targets (and repeated targets with the "repeat" op) allows for complex logic-time looping

---
#### ops

- targets
    - `open_target`
        - inputs: `n -> 1`
        - open target white holes when input is non-zero
    - `close_target`
        - inputs: `n -> 1`
        - close target white holes when input is non-zero
    - `open_nth`
        - inputs: `n -> 1`
        - open nth target once if it's a white hole
    - `del_target`
        - inputs: `n -> 1`
        - delete targets and clear targets array when input is non-zero
    - `spin_target`
        - inputs: `n`, `n -> 1`
        - rotate targets around self by self `n` when input `n` is non-zero
    - `reorder`
        - inputs: `n -> 1`
        - set target circles' order to input `n`
    - `spawn`
        - inputs: `n -> 1`
        - spawn a new circle similar to self when input is non-zero
    - `distro`
        - inputs: `A -> n/r/x/y/z/r/o/v/h/s/l/a`
        - distribute values from input array among targets
- arrays
    - `repeat`
        - inputs: `n -> 1` (repetitions), `A -> 2` or `T -> 2`
        - repeat input array (or input targets array) n times 
    - `zip`
        - inputs: `A -> 1`, `A -> 2`
        - zip array 1 and array 2
    - `unzip`
        - inputs: `A -> 1`
        - unzip input array (one side remains in input array, the other side is in self)
    - `push`
        - inputs: `n -> 1`
        - push input num to self's array
    - `pop`
        - inputs: `n -> 1`
        - pop the last number in the array and set self's num to it when input is non-zero
    - `len`
        - inputs: `A -> 1`
        - length of input array
    - `append`
        - inputs: `A -> 1`
        - copy input array and append it to the end of self's array
    - `slice`
        - inputs: `n`, `A -> 1`
        - slice input array at index `n`, [0..n] remain in input array, [n..len] are moved to self's array
    - `resize`
        - inputs: `n -> 1`
        - resize self's array, discards numbers when shrinking, and adds zeros when growing
    - `contains`
        - inputs: `A -> 1`, `n -> 2`
        - outputs 1 when input array contains input num, 0 otherwise
    - `set`
        - inputs: `n -> 1`, `n -> 2`
        - first input is index, second is value. sets the value of the given index of self's array
    - `get`
        - inputs: `A -> 1`, `n -> 2`
        - get the value at index of the input array
    - `collect`
        - inputs: `n -> {non-negative}` (any number of those)
        - collect all connected nums and create an array of them in order (in self)
- settings
    - `clear_color`
        - when color changes (drag h/s/l), sets the background color (the clear color)
    - `draw_verts`
        - when vertices change, set the default drawing vertices for future circles
    - `draw_color`
        - when color changes, set the default drawing color
    - `highlight_color`
        - when color changes, set the highlight color (the outline around selected entities)
    - `selection_color`
        - when color changes, set the color of the selection lasso circle (and draw mode indicator)
    - `connection_color`
        - when color changes, set the color of connection arrows
    - `connecting_line_color`
        - when color changes, set the connect mode indicator
    - `command_color`
        - when color changes, set color of the command line text
    - `tonemapping`
        - inputs: `n -> 1`
        - input num sets the tonemapping mode. 0 = `None`, 1 = `Reinhard`, 2 = `ReinhardLuminance`, 3 = `AcesFitted`, 4 = `AgX`, 5 = `SomewhatBoringDisplayTransform`, 6 = `TonyMcMapface`, 7 = `BlenderFilmic`
    - `bloom`
        - control bloom parameters
        - inputs:
            - `n -> 1` : intensity
            - `n -> 2` : low frequency boost
            - `n -> 3` : low frequency boost curvature
            - `n -> 4` : high pass frequency
            - `n -> 5` : composite mode (if n > 0 `Additive` else `EnergyConserving`)
            - `n -> 6` : prefilter threshold
            - `n -> 7` : prefilter threshold softness
- utils
    - `cam`
        - `n -> 1` camera x position
        - `n -> 2` camera y position
        - `n -> 3` camera z position (can be useful if you're playing with extremes in depth)
        - `n -> 4` camera rotation
        - `n -> 5` zoom
    - `update_rate`
        - inputs: `n -> 1`, `n -> 2`
        - by default quartz will respond (as fast as possible) to any mouse input/movement, or keyboard input, or if the refresh duration has elapsed. that duration is by default 1/60 of a second (60fps) when the window is in focus, and 30fps when out of focus. first input is the refresh rate (in hz) for focused mode, second input is unfocused rate
    - `screenshot`
        - inputs: `n -> 1`
        - when input num is non-zero, take a screenshot and save it as screenshots/{time in ms since 1970}.png (make sure that folder exists)
- input
    - `mouse`
        - array stores mouse position (in world coordinates) [x, y]
    - `lmb_pressed`
        - num = 1 if left mouse button is pressed, 0 otherwise
    - `mmb_pressed`
        - num = 1 if middle mouse button is pressed, 0 otherwise
    - `rmb_pressed`
        - num = 1 if right mouse button is pressed, 0 otherwise
    - `butt`
        - num = 1 when clicked, 0 otherwise
    - `toggle`
        - num = 1 when clicked, 0 when clicked again (kinda)
    - `key`
        - pressed keyboard keys are added to this circle's array and removed when released. for keys corresponding to an ascii character that's their decimal [ascii](https://en.wikipedia.org/wiki/ASCII#Control_code_chart) code, for other keys it's an arbitrary convention that i put together in 5 minutes:
            - Control: 128, Shift: 129, Alt: 130, Super: 131, Fn: 132
            - CapsLock: 133, NumLock: 134, ScrollLock: 135
            - End: 136, Home: 137, PageUp: 138, PageDown: 139
            - Insert: 140, ContextMenu: 141
            - ArrowUp: 200, ArrowDown: 201, ArrowLeft: 202, ArrowRight: 203
            - F1: -1, F2: -2 .. F12: -12
- data management xD
    - `process`
        - [`process`](#process)
    - `apply`
        - inputs: `0 -> 1` (input audio node), `A -> 2` (input array)
        - process the input array as input to the given audio node (array length must match the number of input channels the node has) output of the node is written to this circle's array (process one audio frame)
    - `render`
        - inputs: `0 -> 1`, `n -> 2`
        - render n samples from the given audio node into the array (node must have 0 inputs, and only first channel's output is saved)
    - `rise`
        - inputs: `n -> 1`
        - num = 1 when there's a rise in the input num (current input > previous input), 0 otherwise (uses the array to store previous value)
    - `fall`
        - inputs: `n -> 1`
        - num = 1 when there's a fall in the input num, 0 otherwise (same)
    - `store`
        - inputs: `n -> 1`
        - store the input num into self's num, but doesn't open the white holes reading nums like usual
    - `sum`
        - inputs: `n -> 1` (any number of those)
        - convenience op for adding numbers together
    - `product`
        - inputs: `n -> 1` (any number of those)
        - multiply numbers together
    - `count`
        - inputs: `n -> 1`, [`n -> 2`]
        - count up by first input. if second input is connected, count will wrap around that given number
- audio node management (refer to the fundsp [readme](https://github.com/SamiPerttu/fundsp), and [docs](https://docs.rs/fundsp/latest/fundsp/) for more details)
    - `var()`
        - node: 0 ins, 1 out
        - create a shared variable audio node. its output is the value of this circle's num
    - `monitor()`
        - node: 1 in, 1 out (it passes audio through)
        - create a monitor node. sets the value of this circle's num to the latest sample that passed through this node
    - `timer()`
        - this one's weird (might delete later) (has to be stacked with another node and sets self's num to the time..
    - `get()`
        - custom node: 1 in (index), 1 out (value)
        - copies this circle's array into node so it can be indexed at audio-rate. input is index, output is the value at that index
    - `feedback()`
        - inputs: `0 -> 1` (input node), [`n -> 2`] (optional delay)
        - mixes outputs of given node back into its inputs (number of node ins/outs must match)
        - node: ins and outs are the same as the input node
    - `seq()`
        - inputs: `0 -> {non-negative}` (any number of those)
        - node: 4 ins (trig, node index, delay, duration), 1 out (output from sequenced nodes)
        - sequences the given nodes and mixes their outputs at output (valid input nodes must have no inputs, and only one output). for every sample trig is non-zero, add an event for the node at index with the given delay and duration (in seconds, rounded to nearest sample)
        - indexes are collected. e.g. if circle has three connections: `0 -> 1` `0 -> 5` `0 -> 8` this is gonna be a sequencer node that accepts indexes 0, 1, and 2. the node at 1 at index 0, the node at 5 at index 2, etc. and only valid nodes are added.
    - `select()`
        - inputs: `0 -> {non-negative}` (any number of those)
        - node: 1 in (index of selected node), 1 out (output from that node)
        - create a node that switches between input nodes based on index
    - `wave()`
        - inputs: `A -> 1`
        - node: 0 ins, 1 out
        - create a wave player from the input array
    - `+` `SUM`
        - inputs: `0 -> {non-negative}` (any number of those)
        - sum given nodes together. their number of outputs must match, their inputs are stacked together in the order they appear in connections
    - `*` `PRO`
        - inputs: `0 -> {non-negative}` (any number of those)
        - multiply given nodes together. their number of outputs must match, their inputs are stacked together in the order they appear in connections
    - `-` `SUB`
        - inputs: `0 -> 1`, `0 -> 2`
        - node 1 - node 2 (number of outputs of those nodes must match)
    - `>>` `PIP`
        - inputs: `0 -> {non-negative}` (any number of those)
        - pipe nodes though each other. if outputs of node 1 matches inputs of node 2 they're piped together, and so on
    - `|` `STA`
        - inputs: `0 -> {non-negative}` (any number of those)
        - stack inputs and outputs of given nodes
    - `&` `BUS`
        - inputs: `0 -> {non-negative}` (any number of those)
        - bus given nodes together. number of inputs and outputs must match. input is passed through each node and output from them is mixed at output
    - `^` `BRA`
        - inputs: `0 -> {non-negative}` (any number of those)
        - branch given nodes together (same inputs are passed to each node, but their outputs are kept separate)
    - `!` `THR`
        - inputs: `0 -> 1`
        - pass extra inputs through
    - `branch()`
        - inputs: `A -> 1`, `0 -> 2`
        - create as many nodes as the input array has values, replacing the "#" in the second input's op with each value, all branched together. e.g. array: [1, 2, 3] and op string "lowpass(1729, #)" creates the node `lowpass(1729, 1) ^ lowpass(1729, 2) ^ lowpass(1729, 3)`
    - `bus()`
        - same as branch() but bus nodes together instead
    - `pipe()`
        - same as branch() but pipe nodes together
    - `stack()`
        - same as branch() but stack nodes
    - `sum()`
        - same as branch() but sum
    - `product()`
        - same as branch() but
    - `out()` `dac()`
        - inputs: `0 -> 1`
        - output given node to speakers (node must have 1 or 2 outputs)

<details><summary>audio nodes</summary>
<p>

    - `shift_reg()` 2 ins (trigger signal, input signal), 8 outs (outputs of the shift register)
    - `meter(peak/rms, float)`
    - `sink()`
    - `pass()`
    - `panner()`
    - `pulse()`
    - `brown()`
    - `pink()`
    - `white()` `noise()`
    - `allpole()`
    - `lorenz()`
    - `mls()`
    - `pinkpass()`
    - `rossler()`
    - `tick()`
    - `zero()`
    - `impulse()`
    - `pan(float)`
    - `sine([float])`
    - `saw([float])`
    - `square([float])`
    - `triangle([float])`
    - `organ([float])`
    - `add(float, [float], [float], ...)` (up to 8 params)
    - `sub(float, [float], [float], ...)` (up to 8)
    - `adsr(float, float, float, float)`
    - `allpass([float], [float])` if 1 param is given, that's the q, and the node takes 2 input channels (signal, and hz) if 2 are given, that's the hz and q and the node only takes input signal.. that's a bit verbose
    - `allpole_delay(float)`
    - `bandpass([float], [float])`
    - `bandrez([float], [float])`
    - `bell([float, float], [float])`
    - `biquad(float, float, float, float, float)`
    - `butterpass([float])`
    - `chorus(float, float, float, float)`
    - `clip([float, float])`
    - `constant(float)` `dc(float)`
    - `dc_block([float])`
    - `declick([float])`
    - `delay(float)`
    - `dsf_saw([float])`
    - `dsf_square([float])`
    - `fir(float [float], [float], ...)` (up to 10 weights)
    - `fir3(float)`
    - `follow(float, [float])`
    - `hammond([float])`
    - `highpass([float], [float])`
    - `highpole([float])`
    - `highshelf([float, float], [float])`
    - `hold([float], [float])`
    - `join(float)`
    - `split(float)`
    - `reverse(float)`
    - `limiter(float, [float])`
    - `limiter_stereo(float, [float])`
    - `lowpass([float], [float])`
    - `lowpole([float])`
    - `lowrez([float], [float])`
    - `lowshelf([float, float], [float])`
    - `mls_bits(float)`
    - `moog([float], [float])`
    - `morph([float, float, float])`
    - `mul(float, [float], [float], ...)` (up to 8 params)
    - `div(float, [float], [float], ...)` (up to 8 params)
    - `notch([float], [float])`
    - `peak([float], [float])`
    - `pluck(float, float, float)`
    - `resonator([float, float])`
    - `reverb_stereo(float, [float], [float])`
    - `soft_saw([float])`
    - `tap(float, float)`
    - `tap_linear(float, float)`
    - `rotate(float, float)`
    - `t()`
    - `xd()`
    - `xD()`
    - `ar()`
    - `ramp()`
    - `clock()`
    - `rise()`
    - `fall()`
    - `>([float])`
    - `<([float])`
    - `==([float])`
    - `!=([float])`
    - `>=([float])`
    - `<=([float])`
    - `min([float])`
    - `max([float])`
    - `pow([float])`
    - `mod([float])` `rem([float])`
    - `log([float])`
    - `bitand([float])`
    - `bitor([float])`
    - `bitxor([float])`
    - `shl([float])`
    - `shr([float])`
    - `lerp([float, float])`
    - `lerp11([float, float])`
    - `delerp([float, float])`
    - `delerp11([float, float])`
    - `xerp([float, float])`
    - `xerp11([float, float])`
    - `dexerp([float, float])`
    - `dexerp11([float, float])`
    - `abs()`
    - `signum()`
    - `floor()`
    - `fract()`
    - `ceil()`
    - `round()`
    - `sqrt()`
    - `exp()`
    - `exp2()`
    - `exp10()`
    - `exp_m1()`
    - `ln_1p()`
    - `ln()`
    - `log2()`
    - `log10()`
    - `sin()`
    - `cos()`
    - `tan()`
    - `asin()`
    - `acos()`
    - `atan()`
    - `sinh()`
    - `cosh()`
    - `tanh()`
    - `asinh()`
    - `acosh()`
    - `atanh()`
    - `squared()`
    - `cubed()`
    - `dissonance()`
    - `dissonance_max()`
    - `db_amp()`
    - `amp_db()`
    - `a_weight()`
    - `m_weight()`
    - `spline()`
    - `spline_mono()`
    - `soft_sign()`
    - `soft_exp()`
    - `soft_mix()`
    - `smooth3()`
    - `smooth5()`
    - `smooth7()`
    - `smooth9()`
    - `uparc()`
    - `downarc()`
    - `sine_ease()`
    - `sin_hz()`
    - `cos_hz()`
    - `sqr_hz()`
    - `tri_hz()`
    - `semitone_ratio()`
    - `rnd()`
    - `rnd2()`
    - `spline_noise()`
    - `fractal_noise()`

</p>
</details>

i hope that everyone will become friends

## thanks
- tools / dependencies:
    - rust https://github.com/rust-lang/rust
    - bevy https://github.com/bevyengine/bevy
    - bevy_pancam https://github.com/johanhelsing/bevy_pancam
    - bevy-inspector-egui https://github.com/jakobhellermann/bevy-inspector-egui/
    - fundsp https://github.com/SamiPerttu/fundsp
    - cpal https://github.com/rustaudio/cpal
    - copypasta https://github.com/alacritty/copypasta
    - serde https://github.com/serde-rs/serde
    - bevy_github_ci_template https://github.com/bevyengine/bevy_github_ci_template
    - tracy https://github.com/wolfpld/tracy
    - vim https://github.com/vim/vim
    - void linux https://voidlinux.org/
- learning / inspiration / used for a while:
    - modulus salomonis regis https://github.com/AriaSalvatrice/AriaModules
    - network https://github.com/JustMog/Mog-VCV
    - csound https://csound.com
    - faust https://faust.grame.fr
    - plugdata https://github.com/plugdata-team/plugdata
    - bevy-cheatbook: https://github.com/bevy-cheatbook/bevy-cheatbook
    - bevy best practices https://github.com/tbillington/bevy_best_practices
    - knyst https://github.com/ErikNatanael/knyst
    - shadplay https://github.com/alphastrata/shadplay
    - rust-ants-colony-simulation https://github.com/bones-ai/rust-ants-colony-simulation
    - bevy_fundsp https://github.com/harudagondi/bevy_fundsp
    - bevy_kira_audio https://github.com/NiklasEi/bevy_kira_audio
    - crossbeam https://github.com/crossbeam-rs/crossbeam
    - bevy_mod_picking https://github.com/aevyrie/bevy_mod_picking/
    - bevy_vector_shapes https://github.com/james-j-obrien/bevy_vector_shapes
    - anyhow https://github.com/dtolnay/anyhow
    - assert_no_alloc https://github.com/Windfisch/rust-assert-no-alloc

## license

quartz is free and open source. all code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

### your contributions

unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## devlog

https://www.youtube.com/playlist?list=PLW3qKRjtGsGZC7V4eKU_tNwszVZAYvKow

## donate if you can

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/auaudio)

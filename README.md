# quartz

```
"you africans, please listen to me as africans
and you non-africans, listen to me with open mind"
```
![Screenshot_2024-04-28_19-32-55](https://github.com/tomara-x/quartz/assets/86204514/a69b9a7c-396a-44ad-9c40-1702df524a68)
![Screenshot_2024-03-31_04-11-18](https://github.com/tomara-x/quartz/assets/86204514/5b43d686-4e55-4025-b342-502cafaac534)
![Screenshot_2024-03-31_04-12-09](https://github.com/tomara-x/quartz/assets/86204514/8dca28da-788e-49ca-81aa-1ec9c4dad9fd)
![Screenshot_2024-05-31_06-28-48](https://github.com/tomara-x/quartz/assets/86204514/3aebd832-0510-48db-82b7-e26a929fdbfa)

## let's play

### tutorial
this readme is like a reference, and there's some exapmles of the basic ideas here: https://github.com/tomara-x/quartz/discussions/categories/e

---

### building
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
### modes
when you open quartz, it will be an empty window. there's 3 modes:
- **edit**: (default) interact with entities and execute commands (press `e` or `esc`)
- **draw**: draw new circles (press `d`)
- **connect**: connect circles (press `c`)
    - **target**: target an entity from another (hold `t` in connect mode)
---
### circle anatomy
first, define your terms:
- circle: an object that you create in draw mode (they're regular polygons)
- hole: a connection object. these always come in (black hole - white hole) pairs
- entity: i'll use that to refer to any object (circle or hole)

in addition to the shared properties that both circles and holes have (position, color, radius, vertices) a circle holds other things:
- a number (just a float)
- an [op string](#ops): defining what that circle does
     - for example: `sum`, `toggle`, `screenshot`, `lowpass()`
- an array of numbers: for different uses (don't worry it's empty by default, thus allocating no memory)
- an array of [target](#targets) entities that this circle controls in some way (empty by default too)
- an [order](#order) number: defining if/when that circle is processed
- an audio node (defined by the op string) (`dc(0)` by default)
---
### commands
there are 2 types of commands:

1. return-terminated
(type it then press enter) (you can separate them with `;` to run more than one at once)

<details><summary>scene saving/loading</summary>
<p>

- `:e {file name}` edit (open) a scene file (in the assets path) (no spaces)
- `:w {file name}` write (save) a scene file (same)

```
:w moth.cute    // saves the current scene as the file "assets/moth.cute" (OVERWRITES)
:e soup.fun     // opens the file "assets/soup.fun" if it's there
```
(dragging and dropping scene files into a window also works)

</p>
</details>

<details><summary>set values</summary>
<p>

- `:set n [id] {float}` set num value
- `:set r [id] {float}` set radius (use rx or ry to set those independently)
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

</p>
</details>

<details><summary>other</summary>
<p>

- `:lt [id] {link type}` set [link type](#link-types) of selected holes (use shortcut `l`)
- `:dv {float}` set default number of vertices of drawn circles
- `:dc {float} [float] [float] float]` set default color of drawn circles (h s l a)
- `:ht {id}` toggle open a white hole (by id)
- `:q` exit (don't combine with other commands using `;`)

</p>
</details>

note: using the [std constants](https://doc.rust-lang.org/std/f32/consts/index.html) in the commands works. e.g. `:set op dc(-PI)`, `:set n TAU`

2. immediate commands
(these execute when you finish typing them)

<details><summary>drag modes</summary>
<p>

(what happens when dragging selected entities, or when arrow keys are pressed)
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
 
</p>
</details>

<details><summary>shortcuts</summary>
<p>

- `o` shortcut for `:set op `
- `l` shortcut for `:lt `

</p>
</details>

<details><summary>info texts</summary>
<p>

- `II` spawn info texts for selected entities
- `IC` clear info texts
- `ID` show/hide entity id in visible info texts

</p>
</details>

<details><summary>inspect commands</summary>
<p>

(information about the selected entities)
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

</p>
</details>

<details><summary>audio node info</summary>
<p>

- `ni` number of inputs
- `no` number of outputs
- `np` info about the node

</p>
</details>

<details><summary>selection</summary>
<p>

- `sa` select all
- `sA` deselect all
- `sc` select all circles
- `sC` deselect circles
- `sh` select all holes
- `sH` deselect holes
- `sv` select visible entities (in view)
- `sV` deselect visible entities
- `sg` select holes of the selected circles
- `st` select targets of the selected circles
- `<delete>` delete selected entities
- `yy` copy selection to clipboard
- `p` paste copied

note: when drag-selecting, holding `alt` will only select circles (ignores holes), holding `ctrl` will only select holes (ignores circles), and holding `shift` will add to the selection

</p>
</details>

<details><summary>visibility</summary>
<p>

- `vc` toggle circle visibility
- `vb` toggle black hole visibility
- `vw` toggle white hole visibility
- `va` toggle arrow visibility
- `vv` show all

</p>
</details>

<details><summary>other</summary>
<p>

- `<F11>` toggle fullscreen
- `ht` toggle white hole open status
- `F` freeze the command line (press `esc`, `Enter`, or `Backspace` to reactivate it)
- `quartz` shhh!
- `awa` [awawawa](https://www.youtube.com/watch?v=LLrIGJEz818)

</p>
</details>

---
### link types
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

use `}`/`{` to increment/decrement link types. or use `l` with selected holes to set a specific link type

---
### order
every circle has an order (0 or higher). things in order 0 do nothing.

each frame, every circle with a positive order gets processed (does whatever its op defines)
this processing happens.. you guessed it, in *order*

we process things breadth-first. so when a circle is processed, all its inputs must have their data ready (they processed in this frame) to make sure that's the case, their order has to be lower than that of the circle reading their data...

lower order processes first, and the higher the order, the later that circle processes (within the same frame)

unless...

---
### targets

a circle has an array of "targets" those are just entity id's. so it's like a "pointer" to another circles or hole. think of it as a one-way wireless connection.
some ops make a circle do things to its targets. like `process`, `del_targets`, `spin_target`, `distro`, `reorder`, (see [ops](#ops) for full list)

(they allow some things that aren't easy through normal processing. since circles read their input when they process, while targets are written to when the controller circle is processed instead)

---
### ops


<details><summary>targets</summary>
<p>

- `process`
    - this circle will process its targets in the order they appear in the targets array. it doesn't matter what order those targets are. even if they're at order 0 (it's preferable they are at 0 so you don't cause unexpected things). so for every frame a circle with a `process` op is processed, it processes all of its targets in order.
    - you can't nest them. so if a process has another process in its targets, that won't process the second one (to avoid blowing up computers)
- `select_target`
    - input: `n -> 1`
    - select the targets when input is non-zero, deselect them when it's zero
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
    - spawn a new circle similar to self when input is non-zero. the new circle is added to this circle's targets. only the color, vertices, and transform (ish) are copied (z depth is increased with each one)
- `distro`
    - inputs: `A -> n/r/x/y/z/r/o/v/h/s/l/a`
    - distribute values from input array among targets
- `connect_target`
    - inputs: `n -> 1`, [`T -> 2`]
    - remove holes from targets array, then connect each target circle to the next. if array contains 2 numbers they will be used as the connection type (otherwise `0 -> 0`) if second input is provided, the white holes created will be added as targets to that circle
- `isolate_target`
    - inputs: `n -> 1`
    - delete all connections target has when input is non-zero
- `target_lt`
    - inputs: `n -> 1`
    - for hole targets, set their link type to input num

</p>
</details>



<details><summary>arrays</summary>
<p>

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

</p>
</details>



<details><summary>settings</summary>
<p>

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
- `connection_width`
    - when this circle's num changes, set the width of the connection arrows
- `connecting_line_color`
    - when color changes, set the connect mode indicator
- `command_color`
    - when color changes, set color of the command line text
- `text_size`
    - when this circle's num changes, set the font size of info texts
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

</p>
</details>



<details><summary>utils</summary>
<p>

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
- `osc`
    - set the settings of osc sender and receiver
    - `n -> 1` receiver port
    - `0 -> 2` op string of the input sets the host ip (ip to send to)
    - `n -> 3` sender port
- `osc_r_{osc address}`
    - receive osc messages into the array of this circle. the `osc` op must be present in this patch and is processing for this to work. the osc messages must be sent to the given osc address and contain floats. you can receive from multiple addresses (space separated)
    - e.g. `osc_r /gyroscope`, `osc_r /touch1 /touch3`
- `osc_s_{osc address}`
    - inputs: `A -> 1`
    - send the input array as an osc message with the given address (to the host and port set by the `osc` op)
    - e.g. `osc_s /space`

for more info about osc: https://opensoundcontrol.stanford.edu/spec-1_0.html

</p>
</details>



<details><summary>input</summary>
<p>

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
        - `Control`: 128, `Shift`: 129, `Alt`: 130, `Super`: 131, `Fn`: 132
        - `CapsLock`: 133, `NumLock`: 134, `ScrollLock`: 135
        - `End`: 136, `Home`: 137, `PageUp`: 138, `PageDown`: 139
        - `Insert`: 140, `ContextMenu`: 141
        - `ArrowUp`: 200, `ArrowDown`: 201, `ArrowLeft`: 202, `ArrowRight`: 203
        - `F1`: -1, `F2`: -2 .. `F12`: -12
- `pressed_{one or more characters}`
    - e.g. `pressed Hi` this circle's num will be set to 1 when either `H` or `i` is pressed, zero otherwise

</p>
</details>



<details><summary>data management xD</summary>
<p>

- `apply`
    - inputs: `0 -> 1` (input audio node), `A -> 2` (input array)
    - process the input array as input to the given audio node (array length must match the number of input channels the node has) output of the node is written to this circle's array (process one audio frame)
- `render`
    - inputs: `n`, `0 -> 1` (input node), `n -> 2` (trigger)
    - render n samples from the given audio node into the array when the second input is non-zero (node must have 0 inputs, and only first channel's output is saved)
- `rise`
    - inputs: `n -> 1`
    - num = 1 when there's a rise in the input num (current input > previous input), 0 otherwise (uses the array to store previous value)
- `fall`
    - inputs: `n -> 1`
    - num = 1 when there's a fall in the input num, 0 otherwise (same)
- `store`
    - inputs: `n -> 1`
    - store the input num into self's num, but doesn't open the white holes reading nums like usual
- `push_num`
    - inputs: `n -> 1`
    - output this circle's num (open all white holes reading it) when the input num in non-zero
- `sum`
    - inputs: `n -> 1` (any number of those)
    - convenience op for adding numbers together
- `product`
    - inputs: `n -> 1` (any number of those)
    - multiply numbers together
- `count`
    - inputs: `n -> 1`, [`n -> 2`]
    - count up by first input. if second input is connected, count will wrap around that given number

</p>
</details>



<details><summary>audio node management</summary>
<p>

refer to the fundsp [readme](https://github.com/SamiPerttu/fundsp), and [docs](https://docs.rs/fundsp/latest/fundsp/) for more details (sometimes)

- `+` `SUM`
    - inputs: `0 -> {non-negative}` (any number of those), `n` (repetitions)
    - sum given nodes together. their number of outputs must match, their inputs are stacked together in the order they appear in connections
- `*` `PRO`
    - inputs: `0 -> {non-negative}` (any number of those), `n` (repetitions)
    - multiply given nodes together. their number of outputs must match, their inputs are stacked together in the order they appear in connections
- `-` `SUB`
    - inputs: `0 -> 1`, `0 -> 2`
    - node 1 - node 2 (number of outputs of those nodes must match)
- `>>` `PIP`
    - inputs: `0 -> {non-negative}` (any number of those), `n` (repetitions)
    - pipe nodes though each other. if outputs of node 1 matches inputs of node 2 they're piped together, and so on
- `|` `STA`
    - inputs: `0 -> {non-negative}` (any number of those), `n` (repetitions)
    - stack inputs and outputs of given nodes
- `&` `BUS`
    - inputs: `0 -> {non-negative}` (any number of those), `n` (repetitions)
    - bus given nodes together. number of inputs and outputs must match. input is passed through each node and output from them is mixed at output
- `^` `BRA`
    - inputs: `0 -> {non-negative}` (any number of those), `n` (repetitions)
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
- `var()`
    - node: 0 ins, 1 out
    - create a shared variable audio node. its output is the value of this circle's num. must have an order >= 1
- `monitor()`
    - node: 1 in, 1 out (it passes audio through)
    - create a monitor node. sets the value of this circle's num to the latest sample that passed through this node. must have an order >= 1
- `timer()`
    - when stacked with another node, this will maintain the current time of that node in this circle's number. must have an order >= 1
- `get()`
    - node: 1 in (index), 1 out (value)
    - copies this circle's array into node so it can be indexed at audio-rate. input is index, output is the value at that index
- `quantize()`
    - inputs: `A -> 1` (array of steps to quantize to. must have at least 2 different values)
    - node: 1 in, 1 out
    - quantize input to the nearest value in the given steps
- `feedback()`
    - inputs: `0 -> 1` (input node), [`n -> 2`] (optional delay)
    - mixes outputs of given node back into its inputs (number of node ins/outs must match)
    - node: ins and outs are the same as the input node
- `kr()`
    - inputs: `n`, `0 -> 1` (input node)
    - node: 0 ins, 1 out
    - tick the input node once every n samples (input node must have 0 ins and 1 out)
- `reset()`
    - inputs: `n`, `0 -> 1` (input node (must have 0 ins, and 1 out))
    - node: 0 ins, 1 out
    - process the input node, but reset it every n seconds (rounded to nearest sample)
- `reset_v()`
    - inputs: `0 -> 1` (input node (must have 0 ins, and 1 out))
    - node: 1 in, 1 out
    - process the input node but reset it every n seconds. n is specified by the input to this node
- `trig_reset()`
    - inputs: `0 -> 1` (input node (must have 0 ins, and 1 out))
    - node: 1 in, 1 out
    - reset the given node whenever the input is non-zero
- `seq()`
    - inputs: `0 -> {non-negative}` (any number of those)
    - node: 4 ins (trig, node index, delay, duration), 1 out (output from sequenced nodes)
    - sequences the given nodes and mixes their outputs at output (valid input nodes must have no inputs, and only one output). for every sample trig is non-zero, add an event for the node at index with the given delay and duration (in seconds, rounded to nearest sample)
    - indexes are collected. e.g. if circle has three connections: `0 -> 1` `0 -> 5` `0 -> 8` this is gonna be a sequencer node that accepts indexes 0, 1, and 2. the node at 1 has index 0, node at 5 has index 1, and node at 8 has index 2. and only valid nodes are added.
- `select()`
    - inputs: `0 -> {non-negative}` (any number of those)
    - node: 1 in (index of selected node), 1 out (output from that node)
    - create a node that switches between input nodes based on index
- `wave()`
    - inputs: `A -> 1`
    - node: 0 ins, 1 out
    - create a wave player from the input array

</p>
</details>



<details><summary>audio nodes</summary>
<p>

refer to the fundsp [readme](https://github.com/SamiPerttu/fundsp), and [docs](https://docs.rs/fundsp/latest/fundsp/) for more details (in some cases)

sources
- `sine([float])` e.g. `sine(440)` has no inputs, and outputs a sine wave at 440Hz. `sine()` takes 1 input (frequency) and outputs sine wave
- `saw([float])` (same)
- `square([float])` (same)
- `triangle([float])` (same)
- `organ([float])` (same)
- `hammond([float])` (same)
- `soft_saw([float])` (same)
- `dsf_saw([float])` `dsf_saw()` takes 2 inputs (frequency, and roughness [0...1]), `dsf_saw(0.5)` takes only a freq input.
- `dsf_square([float])` (same)
- `pulse()` pulse wave oscillator (frequency, and duty cycle [0...1])
- `brown()` brown noise
- `pink()` pink noise
- `white()` `noise()` white noise
- `zero()` silence
- `impulse()` one sample impulse
- `lorenz()`
- `rossler()`
- `constant(float)` `dc(float)`
- `pluck(float, float, float)` (frequency, gain per sec, high freq damping) input is string excitation signal
- `mls([float])`
- `ramp()` ramp from 0 to 1 at input freq (phasor)
- `clock()` simple clock with 50% duty cycle (just a `sine() >> >(0)`)

filters
- `allpole()`
- `pinkpass()`
- `allpass([float], [float])` if 1 param is given, that's the q, and the node takes 2 input channels (signal, and hz) if 2 are given, that's the hz and q and the node only takes input signal
- `allpole_delay(float)`
- `bandpass([float], [float])` (same as allpass)
- `bandrez([float], [float])` (same)
- `bell([float, float], [float])` if 2 params are given, they're (q, gain), if 3 are give, they're (hz, q, gain), if none, the node takes 4 channels (input, hz, q, gain)
- `biquad(float, float, float, float, float)`
- `butterpass([float])` e.g. `butterpass()` takes 2 inputs (signal, and hz). `butterpass(1729)` takes 1 input
- `dc_block([float])` if no param the cutoff is 10Hz
- `fir(float [float], [float], ...)` (up to 10 weights)
- `fir3(float)` param is gain at nyquist
- `follow(float, [float])` attack and release response times
- `highpass([float], [float])` (same as allpass)
- `highpole([float])` (same as butterpass)
- `highshelf([float, float], [float])` (same as bell)
- `lowpass([float], [float])` (same as allpass)
- `lowpole([float])` (same as butterpass)
- `lowrez([float], [float])` (same as allpass)
- `lowshelf([float, float], [float])` (same as bell)
- `moog([float], [float])` (same as allpass)
- `morph([float, float, float])` (hz, q, morph [-1...1] (-1 = lowpass, 0 = peak, 1 = highpass)) if not provided, the node takes 4 inputs (signal, hz, q, morph) 
- `notch([float], [float])` (same as allpass)
- `peak([float], [float])` (same)
- `resonator([float, float])` (hz, bandwidth) if not provided the node takes 3 inputs (signal, hz, bandwidth)

channels
- `sink()` eats an input channel
- `pass()` takes an input channel and passes it unchanged
- `pan([float])` e.g. `pan(0)` pan input (mono to stereo) `pan()` takes 2 inputs (signal, and pan [-1...1])
- `join(float)` float can be [2...8] e.g. `join(8)` takes 8 inputs and averages them into 1 output
- `split(float)` float can be [2...8] `split(8)` takes 1 input and copies it into 8 outputs
- `reverse(float)` float can be [2...8] reverse the order of channels

envelopes (all subsampled at ~2 ms)
- `adsr(float, float, float, float)`
- `xd([float])` (this is just an `exp(-t*input)`)
- `xD([float], [float])` e.g. `xD()` takes 2 inputs (time and curvature) `xD(5)` takes 1 input specifying the decay time with a curvature of 5. `xD(10, 0.5)` is a decay over 10 seconds with a curvature of 0.5.
- `ar([float, float], [float, float])` if there are no params it takes 4 inputs, if there are 2 params they are the curvature of attack and release and the node takes 2 inputs specifying the times, if there are 4 params they are (attack time, attack curvature, release time, release curvature)

other
- `tick()` one sample delay
- `shift_reg()` 2 ins (trigger signal, input signal), 8 outs (outputs of the shift register)
- `meter(peak/rms, float)` e.g. `meter(rms, 0.5)` `rms(peak, 2)`
- `chorus(float, float, float, float)` (seed, separation, variation, mod frequency)
- `declick([float])` e.g. `declick()` 10ms fade in, `declick(2)` 2 second fade in
- `delay(float)` e.g. `delay(2)` 2 second delay
- `hold(float, [float])` e.g. `hold(0.5)` takes 2 inputs (signal, and sampling frequency) with variability 0.5, `hold(150, 0)` takes one input and samples it at 150Hz with variability 0
- `limiter(float, [float])` look ahead limiter. first param is attack time, second is release time (in seconds)
- `limiter_stereo(float, [float])` (same)
- `reverb_stereo(float, [float], [float])` (room size, reverberation time, damping) when damping isn't provided it defaults to 1, time defaults to 5
- `tap(float, float)` (min delay time, max delay time)
- `tap_linear(float, float)` (same)

math
- `add(float, [float], [float], ...)` (up to 8 params)
- `sub(float, [float], [float], ...)` (same)
- `mul(float, [float], [float], ...)` (same)
- `div(float, [float], [float], ...)` (same)
- `rotate(float, float)`
- `t()` time since the node started processing (subsampled every ~2 ms)
- `rise()` one sample trigger when there's a rise in input
- `fall()` same but fall
- `>([float])` e.g. `>()` takes 2 inputs and compares them. `>(3)` takes one input and compares against 3
- `<([float])` (same from this one...)
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
- `shr([float])` (.. all the way to this one)
- `clip([float, float])` e.g. `clip()` takes 1 input and clips to [-1...1], `clip(-5, 5)` clips to [-5...5]
- `wrap(float, [float])` wrap between 2 numbers (or between 0 and x if only one number is given)
- `mirror(float, float)` mirror (wave fold) between two values
- `lerp([float, float])` e.g. `lerp()` takes 3 inputs (a, b, t) `lerp(3,5)` takes one input (t)
- `lerp11([float, float])` (same..)
- `delerp([float, float])`
- `delerp11([float, float])`
- `xerp([float, float])`
- `xerp11([float, float])`
- `dexerp([float, float])`
- `dexerp11([float, float])` (.. same)
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
- `atan2()`
- `hypot()`
- `pol()`
- `car()`
- `deg()`
- `rad()`
- `recip()`
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
    - rosc https://github.com/klingtnet/rosc
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
    - bevy_mod_osc https://github.com/funatsufumiya/bevy_mod_osc

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

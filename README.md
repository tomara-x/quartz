# Quartz

⚠️ under construction ⚠️

![Screenshot_2024-02-21_19-39-17](https://github.com/tomara-x/quartz/assets/86204514/0102a8ff-5c56-41f9-be1a-446d4e1a34d4)

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

#### modes
when you open quartz, it will be an empty window. there's 3 modes:
- **edit**: (default) interact with entities and execute commands (press `e` or `esc`) 
- **draw**: draw new circles (press `d`)
- **connect**: connect circles (press `c`)
    - **target**: target an entity from another (hold `t` in connect mode)

#### commands
##### return-terminated commands
(you can separate commands with `;` to run more than one command at once)
###### file io
- `:e` edit (open) a scene file (in the assets path and without extension)
- `:w` write(save) a scene file (same)

```
:w moth     // will save the current scene as the file "assets/moth.scn.ron" (OVERWRITES)
:e moth     // will try to open the file "assets/moth.scn.ron" if it's there
```

###### set values

these can take an optional entity id
```
:set n 4v0 42  // will set the num of entity 4v0 to 42
:set n 42      // will set the num values of selected entities to 42
```
- `:set n` set the num value of selected entities
- `:set r` set radius
- `:set x` set x position
- `:set y` set y position
- `:set z` set z position (this controls depth. what's in front of what)
- `:set h` set hue value [0...360]
- `:set s` set saturation [0...1]
- `:set l` set lightness [0...1]
- `:set a` set alpha [0...1]
- `:set v` set number of vertices (3 or higher)
- `:set o` or `:set rot` or `:set rotation` set rotation [-pi...pi]
- `:set op` set op (use shortcut `o`)
- `:set ord` or `:set order` set order (use `[` and `]` to increment/decrement order)
- `:set arr` or `:set array` set the array (space separated, no commas!)
- `:set tar` or `:set targets` set targets (space separated id's) (if nothing is selected, the first entity gets the rest of the list as its targets)
- `:tsel` target selected (`:tsel 4v2` sets selected entities as targets of entity 4v2)
- `:push` push a number to the array, or an id to the targets array 
- `:dv` set default number of vertices of drawn circles
- `:dc` set default color of drawn circles

###### other
- `:lt` set link type of hole (use shortcut `l`)
- `:ht` toggle open a white hole (by id)
- `:q` exit

##### immediate commands
###### run mode switching
- `d` go to draw mode
- `c` go to connect mode

###### drag modes
(what happens when dragging selected entities, or when arrow keys are pressed)

exclusive:
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

add a drag mode: (to drag multiple properties at the same time)
- `Et` add translation
- `Er` add radius
- `En` add num
- `Eh` add hue
- `Es` add saturation
- `El` add lightness
- `Ea` add alpha
- `Eo` add rotation
- `Ev` add vertices

###### white hole
- `ht` toggle open status

###### shortcuts
- `o` shortcut for `:set op `
- `l` shortcut for `:lt `

###### info texts
- `II` spawn info texts for selected entities
- `IC` clear info texts
- `ID` show/hide entity id in visible info texts

###### inspect commands
(information about the selected entities)
- `ii` entity id's
- `in` number values
- `ira` radius values
- `ix` x position
- `iy` y position
- `iz` z position
- `ih` hue value
- `is` saturation
- `il` lightness
- `ial` alpha
- `iv` vertices
- `iro` rotation
- `ior` order
- `iop` op
- `iar` array
- `it` targets
- `iL` hole link type
- `iO` white hole open status

audio unit info:
- `ni` number of inputs
- `no` number of outputs
- `np` info about the unit


###### selection
- `sa` select all
- `sc` select all circles
- `sh` select all holes
- `sg` select holes of the selected circles
- `<delete>` delete selected entities
- `yy` copy selection to clipboard
- `p` paste copied

note: when drag-selecting, holding `alt` will only select circles (ignores holes), holding `ctrl` will only select holes (ignores circles), and holding `shift` will add to the selection

###### visibility
- `vc` toggle circle visibility
- `vb` toggle black hole visibility
- `vw` toggle white hole visibility
- `va` toggle arrow visibility
- `vv` show all

###### other
- `quartz` shhh!

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
- `0` usually means audio network (or nothing)
generally a 0 to 0 connection is gonna do nothing, but when connecting networks, the black hole is type 0, and the white hole is type (positive number)


#### order
every circle has an order (0 or higher). things in order 0 do nothing.

each frame, every circle with a positive order gets processed (does whatever its op defines)
this processing happens.. you guessed it, in *order*

we process things breadth-first. so when a circle is processed, all its inputs must have their data ready (they processed in this frame) to make sure that's the case, their order has to be lower than that of the circle reading their data...

lower order processes first, and the higher the order, the later that circle processes (within the same frame)

unless...

#### process
```
todo!();
```
#### ops
```
todo!();
```
#### targets
```
todo!();
```

## thanks
- tools / dependencies:
    - rust https://github.com/rust-lang/rust
    - bevy https://github.com/bevyengine/bevy
    - bevy_pancam https://github.com/johanhelsing/bevy_pancam
    - bevy-inspector-egui https://github.com/jakobhellermann/bevy-inspector-egui/
    - fundsp https://github.com/SamiPerttu/fundsp
    - cpal https://github.com/rustaudio/cpal
    - assert_no_alloc https://github.com/Windfisch/rust-assert-no-alloc
    - copypasta https://github.com/alacritty/copypasta
    - serde https://github.com/serde-rs/serde
    - bevy_github_ci_template https://github.com/bevyengine/bevy_github_ci_template
    - tracy https://github.com/wolfpld/tracy
    - vim https://github.com/vim/vim
    - void linux https://voidlinux.org/
- learning / inspiration / used for a while:
    - network https://github.com/JustMog/Mog-VCV
    - csound https://csound.com
    - faust https://faust.grame.fr
    - plugdata https://github.com/plugdata-team/plugdata
    - bevy-cheatbook: https://github.com/bevy-cheatbook/bevy-cheatbook
    - knyst https://github.com/ErikNatanael/knyst
    - shadplay https://github.com/alphastrata/shadplay
    - rust-ants-colony-simulation https://github.com/bones-ai/rust-ants-colony-simulation
    - bevy_fundsp https://github.com/harudagondi/bevy_fundsp
    - bevy_kira_audio https://github.com/NiklasEi/bevy_kira_audio
    - crossbeam https://github.com/crossbeam-rs/crossbeam
    - bevy_mod_picking https://github.com/aevyrie/bevy_mod_picking/
    - bevy_vector_shapes https://github.com/james-j-obrien/bevy_vector_shapes
    - anyhow https://github.com/dtolnay/anyhow

## license

quartz is free and open source. all code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

### your contributions

unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## donate if you can

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/auaudio)


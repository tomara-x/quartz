it's better if you don't do the following things:

### have more than one `in()` node

in() is supposed to represent the input to quartz (the physical mic input, unless you use routing software like [jack](https://github.com/jackaudio/jack2))
having more than one (both connected to the audio graph goint to `out()`) means that both of them are taking samples from the audio device. problem is, the audio device is putting these samples in one buffer, and now you have two nodes taking samples from that buffer. so it's like you're running faster than the treadmill can go! (if you need the audio from the mic in 2 different spots in the graph, use a `split()`)

also this is true for anything that processes the node, this means `apply` and `render`. you can't connect `in()` (or a graph containing it) both to an `out()` and to a `render` for example. (use `buffin()` and `buffout()` for this)

### have cycles in the audio graph
this won't work! loops are not a good thing to have. we like [dags 🐶️](https://en.wikipedia.org/wiki/Directed_acyclic_graph) around here!
if you find yourself doing something like this thinking that it would work as a counter:

![Screenshot_2024-08-05_14-48-03](https://github.com/user-attachments/assets/ba3e6397-4f9c-4f60-82f1-32ed75a36b0f)

it won't. use feedback() instead, like so: [^1]

![Screenshot_2024-08-05_18-41-04](https://github.com/user-attachments/assets/a0af77ba-c994-4403-9776-66ad000495cb)


[^1]: this counter will go from 1 to 16777216 before stopping due to floating point precision (16777217 isn't a number you can represent in f32).. oh yeah, it starts from 1. pipe it through a tick() to start from 0

when using the sum op this can work as a counter (we're not in the audio graph), but i think it's puny and limited to this application. whereas `feedback()` can do a lot more. you can still use the same counter above and pass it through an `apply`/`render` instead of doing this:

![Screenshot_2024-08-05_14-56-55](https://github.com/user-attachments/assets/499d1e41-4a1a-4534-a6bc-5f1e46202902)

### connecting a graph containing `swap()` to both `out()` and `render()`/`apply()`
please don't! the way `swap()` works is that it sends the net you give it to every receiver. every connective op down the graph that contains a swap will have one of those receivers. but those are just idle (cause they're not processing) so none of them will receive the new node and only the final graph will actually get it. that's the only reason it works, is that only the final point will actually try receiving. when you connect that same graph to a render, you change that whole store. now you have another active point that can, depending on how active your `render` op is, intercept that node. and so the audio graph (connected to `out()`) doesn't update. (this is another place where `buffin()` and `buffout()` save the day)

### pay no attention to the order when `swap()` nodes are involved
yes, most of the time you can pretty much ignore the order and nothing unusual would happen, but sometimes especially when `swap()` is involved you might spend a while wondering why something isn't working when the scene is loaded.. and it's almost always the order

### expect an output outside of the -1...1 range
quartz's output is clipped to this range, anything above 1 is 1, and anything below -1 is -1. infinity/nan are replaced with 0.

### delete an `out()` before disconnecting it
if you want silence, you can't just delete the `out()` circle. disconnecting it will do that. (the logic, but mostly laziness, is that `out()` only does one thing, control what node is connected to the output device. so deleting the circle itself means we have nothing to control what node is connected)

### open scenes while other scenes are already opened
there's no cleaning! when you open a scene the old one will still be there! you can either select everything and press `delete` then load the new scene. or open a new quartz window.

### run at default update rate if you don't need it
by default the update rate is 60hz focused and 30hz unfocused. you can change that to something lower if you aren't doing anything that needs smooth visual updating. try loading the `lowcpu` scene which is an extreme of 0.01hz but still responsive to input. it saves energy, but keep in mind that it can interfere with the things that need immediate updating. `rise`/`fall` can malfunction

### work without saving
there is no undo!

### make cool patches without sharing
come on!


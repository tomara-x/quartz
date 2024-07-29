multichannel audio is everything. any node can have zero or more input or output channels.

a sine oscillator for example has one input (frequency) and one output (sine wave).
some nodes take many channels as input. a lowpass() for example takes a 3 channel input (signal to filter, frequency, q) and outputs one channel (filtered signal)

fundsp (and so quartz as well) provides many connective operators for sticking nodes together in different and interesting ways to create more complicated nodes.

see fundsp's readme for more details about them: https://github.com/SamiPerttu/fundsp?tab=readme-ov-file#operators

(keep in mind, some of those apply to fundsp, but not quartz due to its visual nature, but the main concepts still apply. see the ops section in quartz's readme for a list of what's available)

you can create an audio node by giving a circle a certain op string (see readme for more details)

select a circle, and press `o` in edit mode, then type `lowpass()` then press enter
now you have a lowpass filter:

![Screenshot_2024-04-22_14-25-06](https://github.com/tomara-x/quartz/assets/86204514/2a4b3956-e7e1-4160-8645-3c28357c650e)

with any node selected, type `np` to show info about the connectivity of the node (and its frequency response in some cases) 

![Screenshot_2024-04-22_14-25-29](https://github.com/tomara-x/quartz/assets/86204514/50ab29f0-7e55-46bf-b604-dff6102766f4)

this sine wave oscillator was created with a creation argument, so it's static at 440Hz and takes 0 inputs, and has 1 output:

![Screenshot_2024-04-22_14-26-01](https://github.com/tomara-x/quartz/assets/86204514/e6e94eef-dff0-4b6e-9c50-116e31a2246c)

this lowpass filter has hz and q arguments, so it only takes 1 input and has 1 output

![Screenshot_2024-04-22_14-26-38](https://github.com/tomara-x/quartz/assets/86204514/7f7ff9fa-6caa-4405-a745-99994625dc85)

you can connect them together by "piping" them using the `>>` or `PIP` op

![Screenshot_2024-04-22_14-28-21](https://github.com/tomara-x/quartz/assets/86204514/9287d66e-dab8-4192-8ca3-537846fb7918)

now this `>>` node has both nodes piped, and you can take that resulting node and treat it like any other node (feed it into other nodes, or output it with an `out()`)


https://github.com/tomara-x/quartz/assets/86204514/32d49a16-3d17-4daa-97e6-bb81f0466f89


the white holes link types determine what is piped into what. so in this case 0 (the oscillator) is being piped into 1 (the filter)


what if we wanted to change the frequency while the oscillator is playing?
that's where the `var()` nodes comes in


https://github.com/tomara-x/quartz/assets/86204514/a7b09968-497c-4f4c-84b5-1710a52174f6

always make sure var has at least order 1, it wont be set while in order 0 like other audio nodes


this diagram from [fundsp's readme](https://github.com/SamiPerttu/fundsp) is very useful for visualizing how connective ops work

![](https://raw.githubusercontent.com/SamiPerttu/fundsp/master/operators.png)

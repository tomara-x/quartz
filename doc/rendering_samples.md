suppose you wanted to get individual samples from an audio graph as an array (for whatever reason). the `render` op allows you to do that

![image](https://github.com/tomara-x/quartz/assets/86204514/0d0d1246-9a27-468e-932f-588557344f4e)

let's explore what's happening here

first the graph you use has to have 0 inputs and 1 output (in this example i'm using a simple `pink()`, but any graph with no inputs will work

![image](https://github.com/tomara-x/quartz/assets/86204514/e61a7bb7-e69e-4da0-abac-a2f361dca8af)

the num of the render circle defines how many samples we want to render

![image](https://github.com/tomara-x/quartz/assets/86204514/9607944e-0171-41da-9b2e-2759173203c4)

the second input triggers the rendering when its num is non-zero. here i'm using a rise which will trigger for only one frame, because i don't want it to keep rendering as long as the button is pressed. but if you want continuous rendering then you can give it a constant 1 for example, or use a `toggle`

![image](https://github.com/tomara-x/quartz/assets/86204514/ab89c7a6-180a-48e4-b0d0-ff261b33f408)

combining this with `distro` you can use the output of a graph to do all kinds of things to lots of circles at once (like the scope in the [comfy-anger](https://github.com/tomara-x/quartz/blob/main/assets/comfy-anger) scene) video: https://www.youtube.com/watch?v=G8aXIkVOfNE&t=78s

here i'm using those 5 samples to control the lightness of the target circles
(i used a `lerp11()` because the output of `pink()` is bipolar, and the lightness values are unipolar)


https://github.com/tomara-x/quartz/assets/86204514/47d1d747-946d-4444-a806-a413a6a97137


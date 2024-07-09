go to connect mode (press `c`) then drag from one circle to another:

https://github.com/tomara-x/quartz/assets/86204514/a4eb6054-5f61-46fb-9df2-b57f6febeda4

exit connect mode (press `e`) then select the newly created holes and give them info texts `II` just to see what's happening.. you see that we've created a `0 -> 0` connection. we can change the connection in place by selecting each hole, pressing `l` then typing a link type, then pressing enter.

i've made it a `n -> h` connection (number controlling the hue)
then `n -> r` (number controlling the radius)

https://github.com/tomara-x/quartz/assets/86204514/9b89ce89-5c06-4015-bea2-a9703ae748e5

while in connect mode, you can see the command line displaying `-- CONNECT -- (0 0)` that indicates the type of connection we will make when dragging. this can be changed by pressing `c` and then typing 2 link type characters back to back. here i typed `n` then `o` and you can see that the connection i made after that was indeed of type `n -> o` (number controlling rotation)

https://github.com/tomara-x/quartz/assets/86204514/3af3454a-f8d9-4284-a850-e0f477694dc6

(any unacceptable characters will translate to link type 0)
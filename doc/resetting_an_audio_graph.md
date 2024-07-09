if you need to reset the audio graph for whatever reason (like a blown up filter) you can select any white hole connecting a node in that graph and open that white hole (by typing the `ht` command)

i'm taking the Q of the filter down to less than zero

https://github.com/tomara-x/quartz/assets/86204514/38955476-9058-4c4d-9a53-81d6c1f55f49

an open white hole connecting an audio node will cause the node connected to it to re-clone that node (and all the other inputs), so causing a chain reaction making the whole graph reset


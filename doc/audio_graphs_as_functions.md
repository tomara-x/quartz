you can treat audio graphs as if they were functions, you give it an input and get back an output.
the `apply` op lets you do that

the graph here is a simple `+` with 2 `pass()` connected to it, so as you can see it has 2 inputs, and one output (the sum of those inputs)

![image](https://github.com/tomara-x/quartz/assets/86204514/6e27d87b-c475-4681-83b4-dd4268d54c15)

if i change the array of the second input circle using the command `:set arr 21 21`

![image](https://github.com/tomara-x/quartz/assets/86204514/09552da3-bf98-4d8e-8be8-6636c51e8515)

then inspect the array of the apply circle by selecting it and typing `iar`, you can see that its array is indeed 21+21

![image](https://github.com/tomara-x/quartz/assets/86204514/64b85df4-a099-474e-9756-b17255a07c55)


reopening the white hole will process another sample from the graph (in case you want samples from an oscillator, or noise, or something like that, that's how you do it.
here i'm reopening it to get different samples from a white noise node
(the input circle has an empty array, since white() takes 0 inputs)

https://github.com/tomara-x/quartz/assets/86204514/f0c05dc1-bcf2-4ee0-85db-94e7fa41462d


you can use `collect` and `distro` to change the input and see the output in real time

https://github.com/tomara-x/quartz/assets/86204514/5f486e46-dbd1-4cb4-bb93-503410fbc449

notes:
- this processing happens only once the array changes (the white hole connecting it is open) so it's very cheap
- the input circle's array must match the input number of the given graph, and the output array will contain the same number of elements as the graph has output channels
- i'm only using + for a simple example, but the graph you use can be anything! :smiling_imp: 

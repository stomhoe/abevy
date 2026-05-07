id: elf3
size: (1, 1)
tags: ["elf"]

let lakeness = avg(fnl.lakesmol, fnl.lakebig)
let lakeness = *(lakeness, continentness)

out = idxmax(0.8, lakeness)

[tempgrass] elf4 biomes: [elf=1(30, 3)]
[lake] ""
[] ""

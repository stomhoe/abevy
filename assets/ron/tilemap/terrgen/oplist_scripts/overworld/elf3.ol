id: elf3
size: (1, 1)
tags: ["elf"]

let lakeness = avg(fnl.lakesmol, fnl.lakebig)
let lakeness = *(lakeness, continentness)

out = idxmax(0.8, lakeness)

[tempgrass] biomes: [elf=1.(16, 0)] elf4
[lake] ""
[] ""

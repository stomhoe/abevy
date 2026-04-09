id: elf3
size: (1, 1)
tags: ["elf"]

let lakeness = max(fnl.lakesmol, fnl.lakebig)
let lakeness = *(lakeness, continentness)
let lakeness = *opo(lakeness, beachness)

out = idxmax(0.8, lakeness)

bif elf4 -> [tempgrass] biomes: [elf=1.0(1.0,0.)]
bif "" -> [lake]
bif "" -> []

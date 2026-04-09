id: elf2
size: (1, 1)
tags: ["river_candidate"]
debug: [beach_avg_noise, shore_proximity, beachness, gravelness, inlandness, continentness, elf_diff]

let inlandness = remap(continentness, 0.2, 0.3, 0.0, 1.0)

let shore_proximity = COMPL inlandness * 0.7

let beach_avg_noise = avg(fnl.beachsmol, fnl.beachbig)

let beachness = *(beach_avg_noise, shore_proximity)
let gravelness = *(COMPL beach_avg_noise, shore_proximity)

out = idxmax(0.4, beachness, gravelness)

bif elf3 -> []
bif "" -> [sand] biomes: [beach=1.0(1.0,0.)]
bif "" -> [gravel] biomes: [beach=1.0(1.0,0.)]

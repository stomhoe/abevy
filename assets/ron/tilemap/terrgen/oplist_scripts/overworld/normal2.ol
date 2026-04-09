id: normal2
size: (1, 1)
debug: [beach_avg_noise, shore_proximity, beachness, gravelness, inlandness, continentness, normal_diff]
tags: ["inlandness", "river_candidate"]

let inlandness = remap(continentness, 0.4, 0.6, 0.0, 1.0)
let shore_proximity = COMPL inlandness * 0.76

let beach_avg_noise = avg(fnl.beachsmol, fnl.beachbig)

let beachness = *(beach_avg_noise, shore_proximity)
let gravelness = *(COMPL beach_avg_noise, shore_proximity)

out = idxmax(0.4, beachness, gravelness)

bif normal3 -> []
bif "" -> [sand] biomes: [beach=1.0(1.0,0.)]
bif "" -> [gravel] biomes: [beach=1.0(1.0,0.)]

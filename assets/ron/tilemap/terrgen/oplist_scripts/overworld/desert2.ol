id: desert2
size: (1, 1)
debug: [beach_avg_noise, shore_proximity, beachness, inlandness, continentness]
tags: ["desert", "river_candidate"]

let inlandness = remap(continentness, 0.2, 0.3, 0.0, 1.0)
let shore_proximity = COMPL inlandness * 0.7
let beach_avg_noise = avg(fnl.beachsmol, fnl.beachbig)
let beachness = *(beach_avg_noise, shore_proximity)

out = idxmax(0.4, beachness)

bif desert3 -> []
bif "" -> [sand] biomes: [desert=1.0(1.0,0.)]

id: anta2
size: (1, 1)
debug: [beach_avg_noise, shore_proximity, gravelness, inlandness, continentness]
tags: [anta, ]
let inlandness = remap(continentness, shared_continentness_min, shared_continentness_max, 0.0, 1.0)

let shore_proximity = COMPL inlandness * 0.65
let beach_avg_noise = avg(fnl.beachsmol, fnl.beachbig)
let gravelness = *(beach_avg_noise, shore_proximity)

out = idxmax(0.4, gravelness)

bif anta3 -> []
bif "" -> [dirt]

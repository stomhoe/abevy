id: arctic2
size: (1, 1)
debug: [beach_avg_noise, shore_proximity, gravelness, inlandness, continentness]
tags: [arctic, ]
let inlandness = remap(continentness, 0.4, 1.007, 0.0, 1.0)

let shore_proximity = COMPL inlandness * 0.65
let beach_avg_noise = avg(fnl.beachsmol, fnl.beachbig)
let gravelness = *(COMPL beach_avg_noise, shore_proximity)

out = idxmax(0.4, gravelness)

bif arctic3 -> []
bif "" -> [dirt]

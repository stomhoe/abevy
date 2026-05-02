id: anta2
size: (1, 1)
debug: [beach_avg_noise, shore_proximity, gravelness, inlandness, continentness]
tags: [anta, ]
let inlandness = remap(continentness, shared_continentness_min, shared_continentness_max, 0.0, 1.0)


out = idxmax(inlandness, 0.02)

[] anta3
[] beach_anta

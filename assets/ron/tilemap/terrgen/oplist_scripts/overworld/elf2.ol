id: elf2
debug: [ inlandness, continentness, elf_diff]
tags: []

let inlandness = remap(continentness, shared_continentness_min, shared_continentness_max, 0.0, 1.0)

out = idxmax(inlandness, 0.05)

[] elf3
[] biomes: [beach=1.0(1.0,0.)] beach_gravel

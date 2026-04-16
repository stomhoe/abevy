id: ow_root
roots: [ow]
debug: []

let beach_avg_noise = avg(fnl.beachsmol, fnl.beachbig)
let normal_tect = +(fnl.tect, 0.0)
let elf_tect = +(fnl.tect.s2, 0.0)
let arctic_tect = +(fnl.tect.s3, -0.2)
let desert_tect = +(fnl.tect2, 0.1)

let islanddiff_input_min = 0.4
let islanddiff_input_max = 0.65
let islanddiff_output_min = 0.3
let islanddiff_output_max = 1.0

let normal_diff = islanddiff(0, normal_tect, elf_tect, arctic_tect, desert_tect)
let normal_diff = remap(normal_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)
let elf_diff = islanddiff(1, normal_tect, elf_tect, arctic_tect, desert_tect)
let elf_diff = remap(elf_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)
let arctic_diff = islanddiff(2, normal_tect, elf_tect, arctic_tect, desert_tect)
let arctic_diff = remap(arctic_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)
let desert_diff = islanddiff(3, normal_tect, elf_tect, arctic_tect, desert_tect)
let desert_diff = remap(desert_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)

out = idxmaxislands(0.45, normal_tect, elf_tect, arctic_tect, desert_tect)

bif ocean -> []
bif normal1 -> []
bif elf1 -> []
bif arctic1 -> []
bif desert1 -> []

id: ow_root
debug: []

let normal_tect = +(fnl.tect, 0.0)
let elf_tect = +(fnl.tect.s2, 0.0)
let anta_tect = +(fnl.tect.s3, 0.0)
let desert_tect = +(fnl.tect2, 0.0)


let shared_continentness_min = 0.305
let shared_continentness_max = 1.3


//reajustar estos para cambiar separacion de ISLAS a los bordes de SU PLACA TECTONICA
let islanddiff_input_min = 0.49
let islanddiff_input_max = 0.65


let islanddiff_output_min = 0.0
let islanddiff_output_max = 1.0

let normal_diff = islanddiff(0, normal_tect, elf_tect, anta_tect, desert_tect)
let normal_diff = remap(normal_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)
let elf_diff = islanddiff(1, normal_tect, elf_tect, anta_tect, desert_tect)
let elf_diff = remap(elf_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)
let anta_diff = islanddiff(2, normal_tect, elf_tect, anta_tect, desert_tect)
let anta_diff = remap(anta_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)
let desert_diff = islanddiff(3, normal_tect, elf_tect, anta_tect, desert_tect)
let desert_diff = remap(desert_diff, islanddiff_input_min, islanddiff_input_max, islanddiff_output_min, islanddiff_output_max)

//bajarlo acerca las distintas placas tectonicas
let ocean_between_tectonic_plates = 0.47

out = idxmaxislands(ocean_between_tectonic_plates, normal_tect, elf_tect, anta_tect, desert_tect)

[] ocean
[] normal1
[] elf1
[] anta1
[] desert1

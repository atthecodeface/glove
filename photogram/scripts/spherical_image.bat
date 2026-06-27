# Tetrahedron requires 4 patches
# new_image --name fred --shape tetrahedron
# add_image_file -W data.jpg -width 64 --height 32
# add_toplevel_patches --patch_size 16

# Icosahedron requires 10 patches
# new_image --name fred --shape icosahedron
# add_image_file -W data.jpg -width 80 -height 32
# add_toplevel_patches --patch_size 16

# new_image --name fred --shape tetrahedron
# add_image_file -W data.jpg --width 2048 --height 1024
# add_toplevel_patches --patch_size 1024

# new_image --name fred --shape octahedron
# add_image_file -W data.jpg --width 8192 --height 8192
# add_toplevel_patches --patch_size 4096

# new_image --name fred --shape icosahedron
# add_image_file -W data.jpg --width 20480 --height 8192
# add_toplevel_patches --patch_size 4096

# write_image_file
# --pretty_json json_image --name fred
# echo ${0}

# --batch ./scripts/si_lores_oct.bat
--batch ./scripts/si_hires_oct.bat
# --batch ./scripts/si_midres_oct.bat

# --batch ./scripts/lyon_garden_overlook.bat
# --batch ./scripts/lyon_bridge_at_phenix.bat
--batch ./scripts/lyon_awning_court.bat

write_image_file

# set orientation ' {"LookAt": [[0,0.3,-1], [0.035,1,0]]}'
# --use_body T2i --use_lens 50mm --use_focus 1000000 --verbose render_photo  --write c270.jpg

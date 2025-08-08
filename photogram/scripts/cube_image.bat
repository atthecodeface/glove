# PROJECT=cube_improved.json ./target/release/photogram --batch scripts/cube_image.bat

--path nac --path ../../Images --path ../..
--project_file ${PROJECT} 

named_points list 'M m.*'

set CIP 0

cip --cip ${CIP} image -r ${cip.image} -w IMG_${CIP}_pm.png --pms_color #ff0000 --model_color #00ff00
# cip --cip ${CIP} create_rays_from_camera -r ${cip.image} -w IMG_PATCH_${CIP}.png --np "M m.*"

cip --cip 0 list 'XXX.*'
cip image_patch -r ${cip.image} -w IMG_PATCH_0.png --np "M m.*"

cip --cip 1 list 'XXX.*'
cip image_patch -r ${cip.image} -w IMG_PATCH_1.png --np "M m.*"

cip --cip 2 list 'XXX.*'
cip image_patch -r ${cip.image} -w IMG_PATCH_2.png --np "M m.*"

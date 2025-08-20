# PROJECT=cube_improved.json ./target/release/photogram --batch scripts/cube_image.bat

--path nac --path ../../Images --path ../..
-v --project_file ${PROJECT} 

# named_points list

named_points get_model_points  4V3A6040.JPG  4V3A6041.JPG  4V3A6042.JPG 
named_points update_model ${0}

cip --cip 4V3A6040.JPG list 'XXX.*'
cip image -r ${cip.image_filename} -w ${cip.image_filename}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 4V3A6041.JPG list 'XXX.*'
cip image -r ${cip.image_filename} -w ${cip.image_filename}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 4V3A6042.JPG list 'XXX.*'
cip image -r ${cip.image_filename} -w ${cip.image_filename}_pm.png --pms_color #ff0000 --model_color #00ff00


# cip --cip {CIP} image -r ${cip.image_filename} -w IMG_{CIP}_pm.png --pms_color #ff0000 --model_color #00ff00
# cip --cip {CIP} create_rays_from_camera -r {cip.image_filename} -w IMG_PATCH_{CIP}.png --np "M m.*"

cip --cip 4V3A6040.JPG list 'XXX.*'
cip image_patch -r ${cip.image_filename} -w patch_m_0.png    "top mensa e tip"  "M m.*"
cip image_patch -r ${cip.image_filename} -w patch_tex_0.png "bl text" "mr text" "tl text" "5cm ruler" "1 tl game" "text dimension d"
cip image_patch -r ${cip.image_filename} -w patch_game_0.png "0 bl game" "1 tl game" "2 tr game" "br tower game" "mr game" 

cip --cip 4V3A6041.JPG list 'XXX.*'
cip image_patch -r ${cip.image_filename} -w patch_m_1.png    "top mensa e tip" "M m.*"
cip image_patch -r ${cip.image_filename} -w patch_tex_1.png "bl text" "mr text" "tl text" "5cm ruler" "1 tl game" "text dimension d"
cip image_patch -r ${cip.image_filename} -w patch_game_1.png "0 bl game" "1 tl game" "2 tr game" "br tower game" "mr game" 

cip --cip 4V3A6042.JPG list 'XXX.*'
cip image_patch -r ${cip.image_filename} -w patch_m_2.png    "top mensa e tip"  "M m.*"
cip image_patch -r ${cip.image_filename} -w patch_tex_2.png "bl text" "mr text" "tl text" "5cm ruler" "1 tl game" "text dimension d"
cip image_patch -r ${cip.image_filename} -w patch_game_2.png "0 bl game" "1 tl game" "2 tr game" "br tower game" "mr game" 

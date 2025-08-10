# PROJECT=cube_improved.json ./target/release/photogram --batch scripts/cube_image.bat

--path nac --path ../../Images --path ../..
--project_file ${PROJECT} 

named_points list

named_points get_model_points  0 1 2
named_points update_model ${0}

cip --cip 0 list 'XXX.*'
cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 1 list 'XXX.*'
cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 2 list 'XXX.*'
cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00


# cip --cip {CIP} image -r ${cip.image} -w IMG_{CIP}_pm.png --pms_color #ff0000 --model_color #00ff00
# cip --cip {CIP} create_rays_from_camera -r {cip.image} -w IMG_PATCH_{CIP}.png --np "M m.*"

cip --cip 0 list 'XXX.*'
cip image_patch -r ${cip.image} -w patch_m_0.png    "top mensa e tip" "1 tl game" "2 tr game" "M m.*"
cip image_patch -r ${cip.image} -w patch_tex_0.png "bl text" "mr text" "tl text" "5cm ruler" "1 tl game" "text dimension d"
cip image_patch -r ${cip.image} -w patch_game_0.png "0 bl game" "1 tl game" "2 tr game" "br tower game" "mr game" 

cip --cip 1 list 'XXX.*'
cip image_patch -r ${cip.image} -w patch_m_1.png    "top mensa e tip" "1 tl game" "2 tr game" "M m.*"
cip image_patch -r ${cip.image} -w patch_tex_1.png "bl text" "mr text" "tl text" "5cm ruler" "1 tl game" "text dimension d"
cip image_patch -r ${cip.image} -w patch_game_1.png "0 bl game" "1 tl game" "2 tr game" "br tower game" "mr game" 

cip --cip 2 list 'XXX.*'
cip image_patch -r ${cip.image} -w patch_m_2.png    "top mensa e tip" "1 tl game" "2 tr game" "M m.*"
cip image_patch -r ${cip.image} -w patch_tex_2.png "bl text" "mr text" "tl text" "5cm ruler" "1 tl game" "text dimension d"
cip image_patch -r ${cip.image} -w patch_game_2.png "0 bl game" "1 tl game" "2 tr game" "br tower game" "mr game" 

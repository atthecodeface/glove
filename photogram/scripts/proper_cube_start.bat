# ./target/release/photogram --batch scripts/proper_cube_start.bat

--path nac --path ../../Images/proper_cube --path ../..
--camera_db camera_db.json

named_points add "0cm ruler" #ff0000 0.0,0.0,0.0 0.0
named_points add "1cm ruler" #ff0000 0.0,-10.0,0.0 0.0
named_points add "2cm ruler" #ff0000 0.0,-20.0,0.0 0.0
named_points add "3cm ruler" #ff0000 0.0,-30.0,0.0 0.0
named_points add "4cm ruler" #ff0000 0.0,-40.0,0.0 0.0
named_points add "5cm ruler" #ff0000 0.0,-50.0,0.0 0.0
named_points add "6cm ruler" #ff0000 0.0,-60.0,0.0 0.0
named_points add "7cm ruler" #ff0000 0.0,-70.0,0.0 0.0
named_points add "8cm ruler" #ff0000 0.0,-80.0,0.0 0.0
named_points add "9cm ruler" #ff0000 0.0,-90.0,0.0 0.0
named_points add "10cm ruler" #ff0000 0.0,-100.0,0.0 0.0

named_points list

--use_body 5D --use_lens 50mm --use_focus 400

cip add camera_048.JSON 145A3048.JPG pms_048.json
cip add camera_049.JSON 145A3049.JPG pms_049.json
cip add camera_050.JSON 145A3050.JPG pms_050.json
cip add camera_051.JSON 145A3051.JPG pms_051.json
cip add camera_052.JSON 145A3052.JPG pms_052.json
cip add camera_053.JSON 145A3053.JPG pms_053.json
cip add camera_054.JSON 145A3054.JPG pms_054.json
cip add camera_055.JSON 145A3055.JPG pms_055.json
cip add camera_056.JSON 145A3056.JPG pms_056.json
cip add camera_057.JSON 145A3057.JPG pms_057.json
cip add camera_058.JSON 145A3058.JPG pms_058.json
cip add camera_059.JSON 145A3059.JPG pms_059.json
cip add camera_060.JSON 145A3060.JPG pms_060.json
cip add camera_061.JSON 145A3061.JPG pms_061.json
cip add camera_062.JSON 145A3062.JPG pms_062.json
cip add camera_063.JSON 145A3063.JPG pms_063.json
cip add camera_064.JSON 145A3064.JPG pms_064.json
cip add camera_065.JSON 145A3065.JPG pms_065.json
cip add camera_066.JSON 145A3066.JPG pms_066.json
cip add camera_067.JSON 145A3067.JPG pms_067.json
cip add camera_068.JSON 145A3068.JPG pms_068.json
cip add camera_069.JSON 145A3069.JPG pms_069.json
cip add camera_070.JSON 145A3070.JPG pms_070.json
cip add camera_071.JSON 145A3071.JPG pms_071.json
cip add camera_072.JSON 145A3072.JPG pms_072.json
cip add camera_073.JSON 145A3073.JPG pms_073.json
cip add camera_074.JSON 145A3074.JPG pms_074.json
cip add camera_075.JSON 145A3075.JPG pms_075.json
cip add camera_076.JSON 145A3076.JPG pms_076.json
cip add camera_077.JSON 145A3077.JPG pms_077.json
cip add camera_078.JSON 145A3078.JPG pms_078.json
cip add camera_079.JSON 145A3079.JPG pms_079.json
cip add camera_080.JSON 145A3080.JPG pms_080.json
cip add camera_081.JSON 145A3081.JPG pms_081.json

project as_json
echo --file proper_cube_proj.json ${0}

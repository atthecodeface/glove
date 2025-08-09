# ./target/release/photogram --batch scripts/grid_calibrate.bat -v --path nac --path ../../Images 
# --project nac/nac_proj.json cip locate --cip 0 --np '1cm.*' --np '2cm.*' --np '3cm.*' --np '1 tl game'

--path nac --path ../../Images --path ../..
--camera_db camera_db.json

named_points add "0cm ruler" #ff0000 0.0,0.0,0.0
named_points add "1cm ruler" #ff0000 0.0,-10.0,0.0
named_points add "2cm ruler" #ff0000 0.0,-20.0,0.0
named_points add "3cm ruler" #ff0000 0.0,-30.0,0.0
named_points add "4cm ruler" #ff0000 0.0,-40.0,0.0 5.0
named_points add "5cm ruler" #ff0000 0.0,-50.0,0.0 5.0
named_points add "6cm ruler" #ff0000 0.0,-60.0,0.0 5.0
named_points add "7cm ruler" #ff0000 0.0,-70.0,0.0 5.0
named_points add "8cm ruler" #ff0000 0.0,-80.0,0.0 5.0
named_points add "9cm ruler" #ff0000 0.0,-90.0,0.0 5.0
named_points add "10cm ruler" #ff0000 0.0,-100.0,0.0 5.0

named_points add "M middle" #ff0000  67.0,-36.7,94.0
named_points add "M middle 1" #ff0000
named_points add "M middle 2" #ff0000
named_points add "M middle 3" #ff0000
named_points add "M middle 4" #ff0000
named_points add "M middle 5" #ff0000
named_points add "M middle 6" #ff0000

named_points add "0 bl game" #ff0000
named_points add "1 tl game" #ff0000 1.0,-50.0,91.0
named_points add "tl text" #ff0000 0,53.6,90.0
named_points add "2 tr game" #ff0000
named_points add "text mensa logo middle" #ff0000
named_points add "bl text" #ff0000 
named_points add "mr text" #ff0000 
named_points add "br tower game" #ff0000 
named_points add "br text" #ff0000 

named_points add "10000" #ff0000 
named_points add "10001" #ff0000 
named_points add "10003" #ff0000 
named_points add "bl game" #ff0000 
named_points add "10005" #ff0000 
named_points add "mr game" #ff0000 
named_points add "text dimension d" #ff0000 
named_points add "top mensa e tip" #ff0000 

named_points list

--use_body 5D --use_lens 50mm --use_focus 400

cip add camera_0.json 4V3A6040.JPG pms_0.json

point_mappings add "M middle" 4111.0,1182.0 2.0
point_mappings add "M middle 1" 4248.0,1115.0 2.0
point_mappings add "M middle 2" 3810.0,1221.0 2.0
point_mappings add "M middle 3" 4252.0,1017.0 2.0
point_mappings add "M middle 4" 3121.0,1185.0 2.0
point_mappings add "M middle 5" 3915.0,909.0 2.0
point_mappings add "M middle 6" 3900.0,1096.0 2.0
point_mappings add "0 bl game" 3189.8609607436383,3746.651470852737 2.0
point_mappings add "1 tl game" 3161.0,1559.0 2.0
point_mappings add "2 tr game" 4957,1210 2.0
point_mappings add "tl text" 1555.0,1211.0 2.0
point_mappings add "1cm ruler" 2499.0,3480.0 2.0
point_mappings add "2cm ruler" 2660.0,3548.0 2.0
point_mappings add "3cm ruler" 2811.0,3621.0 2.0
point_mappings add "4cm ruler" 2986.0,3691.0 2.0
point_mappings add "5cm ruler" 3139.0,3765.0 2.0
point_mappings add "6cm ruler" 3303.0,3842.0 2.0
point_mappings add "7cm ruler" 3486.0,3920.0 2.0
point_mappings add "8cm ruler" 3664.0,4000.0 2.0
point_mappings add "9cm ruler" 3858.0,4082.0 2.0
point_mappings add "10cm ruler" 4055.0,4166.0 2.0
point_mappings add "0cm ruler" 2347.861215382398,3406.177445043403            2.
point_mappings add "bl text" 1671,3074 2.0
point_mappings add "mr text" 3154,3082 2.0
point_mappings add "text mensa logo middle" 2833,1787 10.0
point_mappings add "br tower game" 4862,2900 5.0
point_mappings add "bl game" 3190,3747 2.0
point_mappings add "mr game" 4930,2020 2.0
point_mappings add "text dimension d" 2310,2430 2.0

cip add camera_1.json 4V3A6041.JPG pms_1.json

point_mappings add "M middle 5" 5333.0,589.0 2.0
point_mappings add "M middle 3" 5482.0,1000.0 2.0
point_mappings add "M middle 6" 4890.0,1068.0 2.0
point_mappings add "M middle 4" 3719.0,1102.0 2.0
point_mappings add "M middle 1" 5316.0,1198.0 2.0
point_mappings add "M middle" 4952.0,1336.0 2.0
point_mappings add "M middle 2" 4522.0,1350.0 2.0
point_mappings add "1 tl game" 3144.0,1928.0 2.0
point_mappings add "2 tr game" 6184.0,1523.0 2.0
point_mappings add "tl text" 2400.0,519.0 2.0
point_mappings add "0cm ruler" 2750.0,3172.0 2.0
point_mappings add "1cm ruler" 2813.0,3332.0 2.0
point_mappings add "2cm ruler" 2877.0,3496.0 2.0
point_mappings add "3cm ruler" 2944.0,3668.0 2.0
point_mappings add "4cm ruler" 3014.0,3847.0 2.0
point_mappings add "0 bl game" 3134.0,4014.0 2.0
point_mappings add "5cm ruler" 3092.0,4032.0 2.0
point_mappings add "6cm ruler" 3167.0,4224.0 2.0
point_mappings add "7cm ruler" 3248.0,4420.0 2.0
point_mappings add "bl text" 2448,2396 10.0
point_mappings add "mr text" 3101,3407 2.
point_mappings add "text mensa logo middle" 2947,1953 10.0
point_mappings add "br tower game" 5774,3327 10.0
point_mappings add "bl game" 3134,4015 10.0
point_mappings add "mr game" 6027,2412 10.0
point_mappings add "text dimension d" 2718,2222 10.0
point_mappings add "top mensa e tip" 3745,484 2.0

cip add camera_2.json 4V3A6042.JPG pms_2.json

point_mappings add "M middle 3" 3339.0,588.0 2.0
point_mappings add "M middle 5" 2713.0,622.0 2.0
point_mappings add "M middle 1" 3541.0,692.0 2.0
point_mappings add "M middle 6" 3217.0,876.0 2.0
point_mappings add "M middle" 3600.0,879.0 2.0
point_mappings add "M middle 2" 3468.0,1088.0 2.0
point_mappings add "M middle 4" 2826.0,1467.0 2.0
point_mappings add "1 tl game" 3739.5,1900.0 2.976356103000819
point_mappings add "2 tr game" 4154,402 2.0
point_mappings add "tl text" 932.0,2439.0 2.0
point_mappings add "10cm ruler" 5834.0,3769.0 2.0
point_mappings add "9cm ruler" 5556.0,3804.0 2.0
point_mappings add "8cm ruler" 5277.0,3842.0 2.0
point_mappings add "7cm ruler" 5005.0,3874.0 2.0
point_mappings add "6cm ruler" 4734.0,3907.0 2.0
point_mappings add "0 bl game" 4473.0,3912.0 2.0
point_mappings add "5cm ruler" 4468.0,3939.0 2.0
point_mappings add "4cm ruler" 4201.0,3969.0 2.0
point_mappings add "3cm ruler" 3943.0,4002.0 2.0
point_mappings add "2cm ruler" 3687.0,4035.0 2.0
point_mappings add "1cm ruler" 3433.0,4065.0 2.0
point_mappings add "0cm ruler" 3187.0,4092.0 2.0
point_mappings add "bl text" 1887,4183          2.0
point_mappings add "mr text" 4244,3335 2.0
point_mappings add "text mensa logo middle" 3340,2300 10.0
point_mappings add "br tower game" 4685,2085 2.0
point_mappings add "bl game" 4472,3912 2.0
point_mappings add "mr game" 4417,1202 2.0
point_mappings add "text dimension d" 2714,3217 2.0
point_mappings add "top mensa e tip" 1759,1497 2.0


project as_json
echo --file cube.json ${0}

cip --cip 0 locate "0cm.*" "10cm.*" "M middle" "tl text"
cip orient

cip --cip 1 locate "0cm.*" "7cm.*" "M middle" "tl text"
cip orient

cip --cip 2 locate "0cm.*" "10cm.*" "M middle" "tl text"
cip orient

named_points get_model_points 0 1 2
# named_points get_model_points --np "^M" --np ".*game" 0 1 2
named_points update_model ${0}

cip --cip 0 locate
cip orient
cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 1 locate
cip orient
cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 2 locate
cip orient
cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00

--pretty_json project as_json
echo --file cube_improved.json ${0}

cip orient

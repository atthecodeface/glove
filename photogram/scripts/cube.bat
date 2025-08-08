# ./target/release/photogram --batch scripts/grid_calibrate.bat -v --path nac --path ../../Images 
# --project nac/nac_proj.json cip locate --cip 0 --np '1cm.*' --np '2cm.*' --np '3cm.*' --np '1 tl game'

--path nac --path ../../Images --path ../..
--camera_db camera_db.json

named_points add '0cm ruler' #ff0000 0.0,0.0,0.0
named_points add '1cm ruler' #ff0000 0.0,-10.0,0.0
named_points add '2cm ruler' #ff0000 0.0,-20.0,0.0
named_points add '3cm ruler' #ff0000 0.0,-30.0,0.0
named_points add '4cm ruler' #ff0000 0.0,-40.0,0.0 5.0
named_points add '5cm ruler' #ff0000 0.0,-50.0,0.0 5.0
named_points add '6cm ruler' #ff0000 0.0,-60.0,0.0 5.0
named_points add '7cm ruler' #ff0000 0.0,-70.0,0.0 5.0
named_points add '8cm ruler' #ff0000 0.0,-80.0,0.0 5.0
named_points add '9cm ruler' #ff0000 0.0,-90.0,0.0 5.0
named_points add '10cm ruler' #ff0000 0.0,-100.0,0.0 5.0

named_points add 'M middle' #ff0000  67.0,-36.7,94.0
named_points add 'M middle 1' #ff0000
named_points add 'M middle 2' #ff0000
named_points add 'M middle 3' #ff0000
named_points add 'M middle 4' #ff0000
named_points add 'M middle 5' #ff0000
named_points add 'M middle 6' #ff0000

named_points add '0 bl game' #ff0000
named_points add '1 tl game' #ff0000 1.0,-50.0,91.0
named_points add '5 tl text' #ff0000 0,53.6,90.0
named_points add '2 tr game' #ff0000

named_points add 10000 #ff0000 
named_points add 10001 #ff0000 
named_points add 10002 #ff0000 
named_points add 10003 #ff0000 
named_points add 10004 #ff0000 
named_points add 10005 #ff0000 
named_points add 10006 #ff0000 
named_points add 10007 #ff0000 
named_points add 10008 #ff0000 
named_points add 10009 #ff0000 

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
point_mappings add "5 tl text" 1555.0,1211.0 2.0
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
point_mappings add "5 tl text" 2400.0,519.0 2.0
point_mappings add "0cm ruler" 2750.0,3172.0 2.0
point_mappings add "1cm ruler" 2813.0,3332.0 2.0
point_mappings add "2cm ruler" 2877.0,3496.0 2.0
point_mappings add "3cm ruler" 2944.0,3668.0 2.0
point_mappings add "4cm ruler" 3014.0,3847.0 2.0
point_mappings add "0 bl game" 3134.0,4014.0 2.0
point_mappings add "5cm ruler" 3092.0,4032.0 2.0
point_mappings add "6cm ruler" 3167.0,4224.0 2.0
point_mappings add "7cm ruler" 3248.0,4420.0 2.0
point_mappings add "10000" 5797.12,3512.3199999999997 2.0
# point_mappings add "10003" 6183.469367758073,1522.331491240953 2.0
point_mappings add "10004" 2401.747162675448,519.8327956970589 2.0
point_mappings add "10005" 1720.32,3324.16 2.0
point_mappings add "10006" 6027.516440150328,2412.7814050462707 2.0
point_mappings add "10007" 2718.3678666292712,2216.0416362586193 2.0
point_mappings add "10008" 3745.233011082475,484.7096750590532 2.0
point_mappings add "10009" 3101.736968930886,3407.563136964671 2.

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
point_mappings add "5 tl text" 932.0,2439.0 2.0
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
point_mappings add "10000" 4728.509421438846,2262.441219639534 2.0
point_mappings add "10001" 6305.673915430725,2608.4229576187545 2.0
point_mappings add "10002" 1887.2831306660732,4183.597206001219          2.0
point_mappings add "10003" 4154.913619622289,402.24381129096946 2.0
point_mappings add "10004" 926.9383869239988,2438.250974300084 2.0
point_mappings add "10006" 4417.851076496167,1202.5348505301693 2.0
point_mappings add "10007" 2714.876033528102,3217.701665089997 2.0
point_mappings add "10008" 1759.1414112278064,1497.6942132992642 2.0
point_mappings add "10009" 4242.588507527505,3333.3553167800023 2.0



cip --cip 0 locate '0cm.*' '10cm.*' 'M middle' '5 tl text'
cip orient
# cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 1 locate '0cm.*' '7cm.*' 'M middle' '5 tl text'
cip orient
# cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00

cip --cip 2 locate '0cm.*' '10cm.*' 'M middle' '5 tl text'
cip orient
# cip image -r ${cip.image} -w ${cip.image}_pm.png --pms_color #ff0000 --model_color #00ff00


named_points get_model_points --np '^M' --np '.*game' 0 1 2
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

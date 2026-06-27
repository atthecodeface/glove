
set lens "50mm"

# Images are 8192 x 5464 so middle is 4096, 2732

# Use central image 3986; images are (clockwise) 84, 85, 86, 88, 89
# Images do not overlap a lot

# Use identity orientation for first image
--camera_db nac/camera_db.json --use_body R5 --use_lens "${lens}" --use_focus 100000

# Note - photo_map_pts *uses* the orientation, but the same one for all points - so for this usage model it is irrelevant
# The use of orientation_mapping_pts is mapping from the one (left pair) to the other (right pair) ON TOP of the camera orientation

# Points are:
#  1. top of closest light pole with circular lights
#  2. top left corner of *middle* chimney on build next to flat roof building
#  3. top of middle light pole with circular lights
#  4. top left corner of second chimney to right of middle light pole with circular lights
#  5. top of further light pole
#  6. pinaccle of right-most small roof on distance palace
#  7. top-left end of wooden support for third X in blue awnings right of the statue
# 84: 1 => 5685,1033; 2 => 7651,1288, 3 => X, 4 => X
# 85: 1 => 1911,978; 2 => 3837,1279, 3 => 6341,1321, 4 => X
# 86: 1 => X, 2 => 4,1091 3 => 2594,1220; 4 => 3224,1748, 5 => 5310,1547, 6 => 7917,1449
# 88: 4 => 14,1628; 5 => 2177,1463, 6 => 4706,1437, 7 => 8092,2252
# 89: 6 => 441,1363; 7 => 3806,2261

set orientation_86 ' {"Quaternion": [0,0,0,1]}'

# Pts 2 and 3
photo_map_pts  --xy [[4,1091],[2594,1220],[3837,1279],[6341,1321]]
set pts "${0}"
set orientation ${orientation_86}
--verbose quaternion_mapping_pts --xyz "${pts}"
set orientation_86_to_85 " \{\"Quaternion\": ${0}\}"

# Pts 1 and 2
# 85 to 84 on top of 86 to 85
photo_map_pts  --xy [[1911,978],[3837,1279],[5685,1033],[7651,1288]]
set pts "${0}"
set orientation ${orientation_86_to_85}
quaternion_mapping_pts --xyz "${pts}"
set orientation_86_to_84 " \{\"Quaternion\": ${0}\}"


--use_focus 1000

# Pts 4 and 5
photo_map_pts  --xy [[3224,1748],[5310,1547],[14,1628],[2177,1463]]
set pts "${0}"
set orientation ${orientation_86}
quaternion_mapping_pts --xyz "${pts}"
set orientation_86_to_88 " \{\"Quaternion\": ${0}\}"

# Pts 6 and 7
photo_map_pts  --xy [[4706,1437],[8092,2252],[441,1363],[3806,2261]]
set pts "${0}"
set orientation ${orientation_86_to_88}
quaternion_mapping_pts --xyz "${pts}"
set orientation_86_to_89 " \{\"Quaternion\": ${0}\}"

set blend 0.0
set blend 0.5

--use_focus 100000

--verbose --orientation ${orientation_86}          read_photo --read /Users/gjstark/Git/Images/Lyon/145A3986.JPG --blend 0.0
--verbose --orientation ${orientation_86_to_85} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3985.JPG --blend ${blend}
--verbose --orientation ${orientation_86_to_84} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3984.JPG --blend ${blend}
--verbose --orientation ${orientation_86_to_88} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3988.JPG --blend ${blend}
--verbose --orientation ${orientation_86_to_89} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3989.JPG --blend ${blend}
--verbose --orientation ${orientation_86}          read_photo --read /Users/gjstark/Git/Images/Lyon/145A3986.JPG --blend ${blend}

# set orientation ' {"LookAt": [[-0.08,0.0,-1], [0.02,1,0]]}'
--orientation ${orientation_86} --verbose render_panorama --width 4096 --height 1200  --fovv 40 --fovh 200 --write panorama.jpg

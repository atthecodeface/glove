
set orientation ' {"Quaternion": [0,0,0,1]}'
set lens "15mm equisolid"

# Images are 8192 x 5464 so middle is 4096, 2732

# Short cathedral spike cross on W side of N transept is at IMG30 5633,2605; IMG29 4193,2492; IMG36 4964,814; IMG28 3447,2566; IMG32 6905,2477; IMG35 5085,1661
# Bright roof corner at almost-centre-of-img30 is at IMG30 4094,2457; IMG29 2648,2472; IMG36 3339,786; IMG28 1921,2643; IMG32 5414,2256; IMG35 3511,1630

--camera_db nac/camera_db.json --orientation ${orientation} --use_body R5 --use_lens "${lens}" --use_focus 1000000

# Points are [img30 pair], [imgX pair]
--verbose photo_map_pts  --xy [[5633,2605],[4096,2457],[3447,2566],[1921,2643]]
set pts "${0}"
--verbose quaternion_mapping_pts --xyz "${pts}"
set orientation_28_from_30 " \{\"Quaternion\": ${0}\}"

--verbose photo_map_pts  --xy [[5633,2605],[4096,2457],[4193,2492],[2648,2472]]
set pts "${0}"
--verbose quaternion_mapping_pts --xyz "${pts}"
set orientation_29_from_30 " \{\"Quaternion\": ${0}\}"

--verbose photo_map_pts  --xy [[5633,2605],[4096,2457],[6905,2477],[5414,2256]]
set pts "${0}"
--verbose quaternion_mapping_pts --xyz "${pts}"
set orientation_32_from_30 " \{\"Quaternion\": ${0}\}"

--verbose photo_map_pts --xy [[5633,2605],[4096,2457],[5085,1661],[3511,1630]]
set pts "${0}"
--verbose quaternion_mapping_pts --xyz "${pts}"
set orientation_35_from_30 " \{\"Quaternion\": ${0}\}"

--verbose photo_map_pts --xy [[5633,2605],[4096,2457],[4964,814],[3339,786]]
set pts "${0}"
--verbose quaternion_mapping_pts --xyz "${pts}"
set orientation_36_from_30 " \{\"Quaternion\": ${0}\}"

set blend 0.0

--verbose --orientation ${orientation}            read_photo --read /Users/gjstark/Git/Images/Lyon/145A3830.JPG --blend 0.0
--verbose --orientation ${orientation_28_from_30} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3828.JPG --blend ${blend}
--verbose --orientation ${orientation_29_from_30} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3829.JPG --blend ${blend}
--verbose --orientation ${orientation_32_from_30} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3832.JPG --blend ${blend}
--verbose --orientation ${orientation_35_from_30} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3835.JPG --blend ${blend}
--verbose --orientation ${orientation_36_from_30} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3836.JPG --blend ${blend}

set orientation ' {"LookAt": [[0,0.3,-1], [0.035,1,0]]}'
--orientation ${orientation} --verbose render_panorama --width 4096 --height 1400  --fovv 80 --fovh 280 --write panorama.jpg

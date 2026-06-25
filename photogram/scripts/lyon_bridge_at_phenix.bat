
set lens "28mm linear"

# Images are 8192 x 5464 so middle is 4096, 2732

# Use central image 3841; images are (clockwise) 39, 40, 41, 42, 43, 38
# Images do not overlap a lot

# Use identity orientation for first image
set orientation ' {"Quaternion": [0,0,0,1]}'
--camera_db nac/camera_db.json --orientation ${orientation} --use_body R5 --use_lens "${lens}" --use_focus 1000

# lens_polys_of_pts --xy [[0.03,0.03],[-0.1,-0.1]]
# echo "${0}"


# Note - photo_map_pts *uses* the orientation, but the same one for all points - so for this usage model it is irrelevant
# The use of orientation_mapping_pts is mapping from the one (left pair) to the other (right pair) ON TOP of the camera orientation

# Points are bottom left corner of left-most window pane above Livity Records; top of tram pole to right of Bistrot Bondy
photo_map_pts  --xy [[540,2827],[3016,2311],[3965,2605],[6382,2114]]
set pts "${0}"
--orientation ${orientation}
quaternion_mapping_pts --xyz "${pts}"
set orientation_41_to_40 " \{\"Quaternion\": ${0}\}"

# Top of tram pole on embankment in front of Elephant and castle IMG40 1820,2325  IMG39 7331,2328
# Top left of flag on side of bridge IMG40 291,2602 IMG39 5851,2551
# 39 to 40 on top of 40 to 41
photo_map_pts  --xy [[1820,2325],[291,2602],[7331,2328],[5851,2551]]
set pts "${0}"
--orientation ${orientation_41_to_40}
quaternion_mapping_pts --xyz "${pts}"
set orientation_41_to_39 " \{\"Quaternion\": ${0}\}"

# top of spike in front of church tower just below and right of drainpipe IMG41 4792,2118; IMG42 1763,1768
# top of tram pole from embankment in front of boundary between pink/yellow building IMG41 7619,2333; IMG42 4427,2087
photo_map_pts  --xy [[4792,2118],[7619,2333],[1763,1768],[4427,2087]]
set pts "${0}"
--orientation ${orientation}
quaternion_mapping_pts --xyz "${pts}"
set orientation_41_to_42 " \{\"Quaternion\": ${0}\}"

# top of tram pole from embankment in front of boundary between pink/yellow building IMG42 4427,2087; IMG43 1077,1933
# Top of traffic light pole other side of river in front of mustard building IMG42 7693,2734 IMG43 4277,2635
photo_map_pts --xy [[4427,2087],[7693,2734],[1077,1933],[4277,2635]]
set pts "${0}"
--orientation ${orientation_41_to_42}
quaternion_mapping_pts --xyz "${pts}"
set orientation_41_to_43 " \{\"Quaternion\": ${0}\}"

# Top of narrow bollard on pavement above steps down on embankment IMG43 4277,2635 IMG38,2991,2519
# Top of narrow bollard on pavement above steps down on embankment IMG43 7326,2637 IMG38,5946,2457
--verbose photo_map_pts --xy [[4277,2635],[7326,2637],[2991,2519],[5946,2457]]
set pts "${0}"
--orientation ${orientation_41_to_43}
quaternion_mapping_pts --xyz "${pts}"
set orientation_41_to_38 " \{\"Quaternion\": ${0}\}"

set blend 0.7
set blend 0.0

--verbose --orientation ${orientation}          read_photo --read /Users/gjstark/Git/Images/Lyon/145A3841.JPG --blend 0.0
--verbose --orientation ${orientation_41_to_40} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3840.JPG --blend ${blend}
--verbose --orientation ${orientation_41_to_39} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3839.JPG --blend ${blend}
--verbose --orientation ${orientation_41_to_42} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3842.JPG --blend ${blend}
--verbose --orientation ${orientation_41_to_38} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3838.JPG --blend ${blend}
--verbose --orientation ${orientation_41_to_43} read_photo --read /Users/gjstark/Git/Images/Lyon/145A3843.JPG --blend ${blend}
--verbose --orientation ${orientation}          read_photo --read /Users/gjstark/Git/Images/Lyon/145A3841.JPG --blend ${blend}

set orientation ' {"LookAt": [[-0.08,0.0,-1], [0.02,1,0]]}'
--orientation ${orientation} --verbose render_panorama --width 4096 --height 1200  --fovv 40 --fovh 200 --write panorama.jpg

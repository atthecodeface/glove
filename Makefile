go: capture
	./capture /dev/video0
	convert -depth 8 -size 640x480 a.gray a.jpg
	./capture /dev/video1
	convert -depth 8 -size 640x480 a.gray b.jpg

track: track_highlights
	./track_highlights /dev/video0 /dev/video1

all: capture track_highlights

capture: capture.c capture_device.c capture_device.h highlights.h highlights.c 
	gcc -o capture capture.c capture_device.c highlights.c

track_highlights: track_highlights.c capture_device.c capture_device.h highlights.h highlights.c 
	gcc -o track_highlights track_highlights.c capture_device.c highlights.c

#                     brightness 0x00980900 (int)    : min=0 max=255 step=1 default=128 value=128
#                       contrast 0x00980901 (int)    : min=0 max=255 step=1 default=32 value=32
#                     saturation 0x00980902 (int)    : min=0 max=255 step=1 default=32 value=32
# white_balance_temperature_auto 0x0098090c (bool)   : default=1 value=1
#                           gain 0x00980913 (int)    : min=0 max=255 step=1 default=64 value=192
#           power_line_frequency 0x00980918 (menu)   : min=0 max=2 default=2 value=2
#      white_balance_temperature 0x0098091a (int)    : min=0 max=10000 step=10 default=4000 value=1690 flags=inactive
#                      sharpness 0x0098091b (int)    : min=0 max=255 step=1 default=24 value=24
#         backlight_compensation 0x0098091c (int)    : min=0 max=1 step=1 default=0 value=0
#                  exposure_auto 0x009a0901 (menu)   : min=0 max=3 default=3 value=3 (1 is manual I believe)
#              exposure_absolute 0x009a0902 (int)    : min=1 max=10000 step=1 default=166 value=254 flags=inactive
#         exposure_auto_priority 0x009a0903 (bool)   : default=0 value=1

# Cannot do this:
CONTROLS = --set-ctrl=power_line_frequency=0 --set-ctrl=saturation=0 --set-ctrl=white_balance_temperature_auto=0 --set-ctrl=white_balance_temperature=2000 --set-ctrl=exposure_auto=1 --set-ctrl=exposure_absolute=40 --set-ctrl=brightness=80 --set-ctrl=contrast=64 --set-ctrl=gain=15

video_controls:
	v4l2-ctl -d /dev/video0 $(CONTROLS)
	v4l2-ctl -d /dev/video1 $(CONTROLS)
	v4l2-ctl -d /dev/video0 -L
	v4l2-ctl -d /dev/video1 -L
